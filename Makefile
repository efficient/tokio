override CARGOFLAGS := --release $(CARGOFLAGS)
override CFLAGS := -std=c99 -O2 $(CFLAGS)
override CPPFLAGS := $(CPPFLAGS)
override LDFLAGS := $(LDFLAGS)
override LDLIBS := $(LDLIBS)

CARGO := cargo
MKDIR := mkdir -p

pngreadc: lib/libpng16.so
pngreadc: private CPPFLAGS += -Ilibpng

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
