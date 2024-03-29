use std::io;
use std::net::{Ipv4Addr, SocketAddr, SocketAddrV4};

use futures::{Async, AsyncSink, Poll, Sink, StartSend, Stream};

use super::UdpSocket;

use bytes::{BufMut, BytesMut};
use tokio_codec::{Decoder, Encoder};

/// A unified `Stream` and `Sink` interface to an underlying `UdpSocket`, using
/// the `Encoder` and `Decoder` traits to encode and decode frames.
///
/// Raw UDP sockets work with datagrams, but higher-level code usually wants to
/// batch these into meaningful chunks, called "frames". This method layers
/// framing on top of this socket by using the `Encoder` and `Decoder` traits to
/// handle encoding and decoding of messages frames. Note that the incoming and
/// outgoing frame types may be distinct.
///
/// This function returns a *single* object that is both `Stream` and `Sink`;
/// grouping this into a single object is often useful for layering things which
/// require both read and write access to the underlying object.
///
/// If you want to work more directly with the streams and sink, consider
/// calling `split` on the `UdpFramed` returned by this method, which will break
/// them into separate objects, allowing them to interact more easily.
#[must_use = "sinks do nothing unless polled"]
#[derive(Debug)]
pub struct UdpFramed<C> {
    socket: UdpSocket,
    codec: C,
    rd: BytesMut,
    wr: BytesMut,
    out_addr: SocketAddr,
    flushed: bool,
    is_readable: bool,
    repeat_decode: bool,
    current_addr: Option<SocketAddr>,
}

impl<C: Decoder> Stream for UdpFramed<C> {
    type Item = (C::Item, SocketAddr);
    type Error = C::Error;

    #[allow(unused_parens)]
    fn poll(&mut self) -> Poll<Option<(Self::Item)>, Self::Error> {
        self.rd.reserve(INITIAL_RD_CAPACITY);

        if self.repeat_decode {
            loop {
                // Are there are still bytes left in the read buffer to decode?
                if self.is_readable {
                    // Use deocde_eof since every datagram contains its own
                    // eof which is just the end of the datagram. This supports
                    // the lines use case where there may not be a terminating
                    // delimiter and thus you may never get the end of the frame.
                    // This is generally fine for most implementations of codec
                    // since by default this will defer to calling decode.
                    if let Some(frame) = self.codec.decode_eof(&mut self.rd)? {
                        trace!("frame decoded from buffer");

                        let current_addr = self
                            .current_addr
                            .expect("will always be set before this line is called");

                        return Ok(Async::Ready(Some((frame, current_addr))));
                    }

                    // if this line has been reached then decode has returned `None`.
                    self.is_readable = false;
                    self.rd.clear();
                }

                // We're out of data. Try and fetch more data to decode
                let (n, addr) = unsafe {
                    // Read into the buffer without having to initialize the memory.
                    let (n, addr) = try_ready!(self.socket.poll_recv_from(self.rd.bytes_mut()));
                    self.rd.advance_mut(n);
                    (n, addr)
                };

                self.current_addr = Some(addr);
                self.is_readable = true;

                trace!("received {} bytes, decoding", n);
            }
        } else {
            let (n, addr) = unsafe {
                // Read into the buffer without having to initialize the memory.
                let (n, addr) = try_ready!(self.socket.poll_recv_from(self.rd.bytes_mut()));
                self.rd.advance_mut(n);
                (n, addr)
            };
            trace!("received {} bytes, decoding", n);
            let frame_res = self.codec.decode(&mut self.rd);
            self.rd.clear();
            let frame = frame_res?;
            let result = frame.map(|frame| (frame, addr)); // frame -> (frame, addr)
            trace!("frame decoded from buffer");
            Ok(Async::Ready(result))
        }
    }
}

impl<C: Encoder> Sink for UdpFramed<C> {
    type SinkItem = (C::Item, SocketAddr);
    type SinkError = C::Error;

    fn start_send(&mut self, item: Self::SinkItem) -> StartSend<Self::SinkItem, Self::SinkError> {
        trace!("sending frame");

        if !self.flushed {
            match self.poll_complete()? {
                Async::Ready(()) => {}
                Async::NotReady => return Ok(AsyncSink::NotReady(item)),
            }
        }

        let (frame, out_addr) = item;
        self.codec.encode(frame, &mut self.wr)?;
        self.out_addr = out_addr;
        self.flushed = false;
        trace!("frame encoded; length={}", self.wr.len());

        Ok(AsyncSink::Ready)
    }

    fn poll_complete(&mut self) -> Poll<(), C::Error> {
        if self.flushed {
            return Ok(Async::Ready(()));
        }

        trace!("flushing frame; length={}", self.wr.len());
        let n = try_ready!(self.socket.poll_send_to(&self.wr, &self.out_addr));
        trace!("written {}", n);

        let wrote_all = n == self.wr.len();
        self.wr.clear();
        self.flushed = true;

        if wrote_all {
            Ok(Async::Ready(()))
        } else {
            Err(io::Error::new(
                io::ErrorKind::Other,
                "failed to write entire datagram to socket",
            )
            .into())
        }
    }

    fn close(&mut self) -> Poll<(), C::Error> {
        try_ready!(self.poll_complete());
        Ok(().into())
    }
}

const INITIAL_RD_CAPACITY: usize = 64 * 1024;
const INITIAL_WR_CAPACITY: usize = 8 * 1024;

impl<C> UdpFramed<C> {
    /// Create a new `UdpFramed` backed by the given socket and codec.
    ///
    /// See struct level documentation for more details.
    pub fn new(socket: UdpSocket, codec: C) -> UdpFramed<C> {
        UdpFramed {
            socket: socket,
            codec: codec,
            out_addr: SocketAddr::V4(SocketAddrV4::new(Ipv4Addr::new(0, 0, 0, 0), 0)),
            rd: BytesMut::with_capacity(INITIAL_RD_CAPACITY),
            wr: BytesMut::with_capacity(INITIAL_WR_CAPACITY),
            flushed: true,
            is_readable: false,
            repeat_decode: false,
            current_addr: None,
        }
    }

    /// Create a new `UdpFramed` backed by the given socket and codec. That will
    /// continue to call `decode_eof` until the decoder has cleared the entire buffer.
    ///
    /// See struct level documentation for more details.
    pub fn with_decode(socket: UdpSocket, codec: C, repeat_decode: bool) -> UdpFramed<C> {
        UdpFramed {
            socket: socket,
            codec: codec,
            out_addr: SocketAddr::V4(SocketAddrV4::new(Ipv4Addr::new(0, 0, 0, 0), 0)),
            rd: BytesMut::with_capacity(INITIAL_RD_CAPACITY),
            wr: BytesMut::with_capacity(INITIAL_WR_CAPACITY),
            flushed: true,
            is_readable: false,
            repeat_decode,
            current_addr: None,
        }
    }

    /// Returns a reference to the underlying I/O stream wrapped by `Framed`.
    ///
    /// # Note
    ///
    /// Care should be taken to not tamper with the underlying stream of data
    /// coming in as it may corrupt the stream of frames otherwise being worked
    /// with.
    pub fn get_ref(&self) -> &UdpSocket {
        &self.socket
    }

    /// Returns a mutable reference to the underlying I/O stream wrapped by
    /// `Framed`.
    ///
    /// # Note
    ///
    /// Care should be taken to not tamper with the underlying stream of data
    /// coming in as it may corrupt the stream of frames otherwise being worked
    /// with.
    pub fn get_mut(&mut self) -> &mut UdpSocket {
        &mut self.socket
    }

    /// Consumes the `Framed`, returning its underlying I/O stream.
    pub fn into_inner(self) -> UdpSocket {
        self.socket
    }
}
