readonly HELPERS="$HELPER client.sh wrk/wrk /lib/x86_64-linux-gnu/ld-linux-x86-64.so.2 /lib/x86_64-linux-gnu/libdl.so.2 /lib/x86_64-linux-gnu/libc.so.6 /lib/x86_64-linux-gnu/libm.so.6 /lib/x86_64-linux-gnu/libpthread.so.0 /usr/lib/x86_64-linux-gnu/libssl.so.1.1 /usr/lib/x86_64-linux-gnu/libcrypto.so.1.1 bimodal.lua"
readonly NCLIENTCORES="14" # client
readonly MSERVERCORES="0xaaaaaaaa" # server

if [ "$#" -ne "2" ]
then
	echo "USAGE: $0 <dest dir> <my ip>"
	exit 1
fi

set -eu
dir="$1" # where to store
addr="$2" # address at which server can be reached

start() {
	taskset "$MSERVERCORES" cargo run --release "$@" &
	await 1337
}

stop() {
	kill "`ps -opid -Chype | tail -n1`" && sleep 2
}

baseline() {
	set -ex
	start >&2
	ssh client "wrk/$HELPER" baseline "$@"
}

overhead() {
	set -ex
	cargo build --release
	cargo build --release --features preemptive -pinger

	export LD_PRELOAD="target/release/libinger.so"
	export LIBGOTCHA_NUMGROUPS="1"
	start >&2
	ssh client "wrk/$HELPER" overhead "$@"
}

preempt() {
	set -ex
	export LIBGOTCHA_NOGLOBALS=
	start --features preemptive >&2
	ssh client "wrk/$HELPER" preempt "$@"
}

await() {
	set +x
	local port="$1"
	while ! fuser "$port/tcp" >/dev/null 2>&1
	do
		:
	done
}

if ! type logindep >/dev/null
then
	logindep() { cat "$@"; }
fi

mkdir "$dir"
./version >"$dir/VERSION"

make -j -Cwrk WITH_OPENSSL=/usr
ssh client rm -rf wrk
ssh client mkdir wrk
unset SSH_AUTH_SOCK
scp $HELPERS client:wrk

stop 2>/dev/null || true

echo "$ $0 $*" >"$dir/LOG"
for indep in $INDEPS
do
	echo "==> INDEP: `echo "$indep" | logindep` <=="
	for series in baseline overhead preempt
	do
		echo "   --> SERIES: $series server <--"
		until
			for file in `"$series" "$indep" "$NCLIENTCORES" "$dir" "$addr"`
			do
				rsync --remove-source-files "client:wrk/$file" "$dir"
			done
			stop
		do
			:
		done
	done
done 2>&1 | tee -a "$dir/LOG"

cp Cargo.lock "$dir/DEPS"

echo "SUCCESS!  Results are in $dir/"