MKDIR := mkdir -p

lib/libpng16.so: libpng/.libs/libpng16.so
	$(MKDIR) $(@D)
	cp $< $@

libpng/.libs/libpng16.so:
	cd libpng && ./configure
	$(MAKE) -Clibpng
