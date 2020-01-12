override BINDFLAGS := --no-layout-tests $(BINDFLAGS)
override CARGOFLAGS := --release $(CARGOFLAGS)
override CFLAGS := -std=c99 -O2 $(CFLAGS)
override CPPFLAGS := $(CPPFLAGS)
override LDFLAGS := $(LDFLAGS)
override LDLIBS := $(LDLIBS)
override RUSTFLAGS := --edition 2018 -O $(RUSTFLAGS)

BINDGEN := bindgen
CARGO := cargo
MKDIR := mkdir -p
RUSTC := rustc

pngreadc: lib/libpng16.so

pngreadrs: png.rs lib/libpng16.so
pngreadrs: private LDLIBS += -lpng16
pngreadrs: private RUSTFLAGS += -Llib

png.rs:
png.rs: private BINDFLAGS += --with-derive-default

lib/libinger.so: libinger/target/release/deps/libinger.so
	$(MKDIR) $(@D)
	cp $(<D)/*.so $(<D)/*.rlib $(@D)

lib/libpng16.so: libpng/.libs/libpng16.so
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
