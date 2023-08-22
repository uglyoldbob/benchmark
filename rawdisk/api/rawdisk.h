#define _CRT_SECURE_NO_WARNINGS
#define WIN32_LEAN_AND_MEAN
#include <windows.h>
#include <stdint.h>

HANDLE open_disk(char *disk);
int64_t read_from_disk(HANDLE h, char *buf, int64_t size);
void close_disk(HANDLE h);