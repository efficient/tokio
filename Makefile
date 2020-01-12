override BINDFLAGS := --no-layout-tests $(BINDFLAGS)
override CARGOFLAGS := --release $(CARGOFLAGS)
override CFLAGS := -std=c99 -O2 $(CFLAGS)
override CPPFLAGS := $(CPPFLAGS)
override LDFLAGS := $(LDFLAGS)
override LDLIBS := $(LDLIBS)
override RUSTFLAGS := --edition 2018 -Llib -O $(RUSTFLAGS)

BINDGEN := bindgen
CARGO := cargo
MKDIR := mkdir -p
RUSTC := rustc

bench: png.rs lib/libtest.rlib lib/libpng16.so
bench: private RUSTFLAGS += --test --extern test=lib/libtest.rlib

pngreadc: lib/libpng16.so

pngreadrs: png.rs lib/libpng16.so

png.rs:
png.rs: private BINDFLAGS += --with-derive-default --raw-line '\#[link(name = "png16")] extern {}'

lib/libinger.so: libinger/target/release/deps/libinger.so
	$(MKDIR) $(@D)
	cp $(<D)/*.so $(<D)/*.rlib $(@D)

lib/libpng16.so: libpng/.libs/libpng16.so
	$(MKDIR) $(@D)
	cp $< $@

lib/libtest.rlib:
lib/libtest.rlib: private RUSTC := RUSTC_BOOTSTRAP= $(RUSTC)

lib/test.rs: libinger/external/libgotcha/test.rs
	$(MKDIR) $(@D)
	cp $< $@

libinger/target/release/deps/libinger.so:
	libinger/configure
	cd libinger && $(CARGO) build $(CARGOFLAGS)

libpng/.libs/libpng16.so:
	cd libpng && ./configure
	$(MAKE) -Clibpng

%: %.rs
	$(RUSTC) $(RUSTFLAGS) $< $(LDLIBS)

%.rs: %.h
	$(BINDGEN) $(BINDFLAGS) -o $@ $<

lib%.rlib: %.rs
	$(RUSTC) --crate-type lib --out-dir $(@D) $(RUSTFLAGS) $< $(LDLIBS)
