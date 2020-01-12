override CARGOFLAGS := --release $(CARGOFLAGS)

CARGO := cargo
MKDIR := mkdir -p

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
