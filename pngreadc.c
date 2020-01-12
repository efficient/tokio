#include "png.h"

#include <sys/mman.h>
#include <sys/stat.h>
#include <sys/time.h>
#include <fcntl.h>
#include <stdint.h>
#include <stdio.h>
#include <stdlib.h>
#include <unistd.h>

enum call {
	_ = 1,
	OPEN,
	FSTAT,
	MMAP,
	MALLOC,
	LEN
};

static int cerror(uintptr_t call) {
	static const char *const NAMES[LEN] = {
		NULL,
		NULL,
		"open()",
		"fstat()",
		"mmap()",
		"malloc()",
	};
	if(call < LEN) {
		perror(NAMES[call]);
		return call;
	} else {
		fprintf(stderr, "png_image_*_read*(): %s\n", ((png_image *) call)->message);
		return LEN;
	}
}

static inline unsigned long usnow(void) {
	struct timeval tv;
	gettimeofday(&tv, NULL);
	return tv.tv_sec * 1000000 + tv.tv_usec;
}

#define return(call) return cerror((uintptr_t) call);

int main(int argc, char **argv) {
	if(argc != 2) {
		printf("USAGE: %s <filename>\n", argv[0]);
		return 1;
	}

	int fd = open(argv[1], O_RDONLY);
	if(fd < 0)
		return(OPEN);

	struct stat st;
	if(fstat(fd, &st))
		return(FSTAT);

	void *src = mmap(NULL, st.st_size, PROT_READ, MAP_SHARED, fd, 0);
	if(src == MAP_FAILED)
		return(MMAP);

	png_image img;
	img.version = PNG_IMAGE_VERSION;
	img.opaque = NULL;

	unsigned long usthen = usnow();
	if(!png_image_begin_read_from_memory(&img, src, st.st_size))
		return(&img);
	printf("begin: %lu us\n", usnow() - usthen);
	img.format = PNG_FORMAT_RGB;

	if(img.warning_or_error)
		fputs(img.message, stderr);

	void *dest = malloc(PNG_IMAGE_SIZE(img));
	if(!dest) {
		png_image_free(&img);
		return(MALLOC);
	}

	usthen = usnow();
	if(!png_image_finish_read(&img, NULL, dest, 0, NULL)) {
		free(dest);
		return(&img);
	}
	printf("finish: %lu us\n", usnow() - usthen);
	free(dest);

	if(img.warning_or_error)
		fputs(img.message, stderr);

	return 0;
}
