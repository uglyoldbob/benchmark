#define _CRT_SECURE_NO_WARNINGS
#define WIN32_LEAN_AND_MEAN
#include <windows.h>
#include <stdint.h>
#include <stdio.h>

HANDLE open_disk(char *disk)
{
    printf("Opening disk -%s-\n", disk);
    HANDLE handle = CreateFile(
        disk,
        GENERIC_READ,
        FILE_SHARE_READ | FILE_SHARE_WRITE,
        NULL, // No security attributes
        OPEN_EXISTING,
        0,
        NULL
    );
    return handle;
}

int64_t read_from_disk(HANDLE h, char *buf, int64_t size, DWORD *bytes_read)
{
    return ReadFile(h, buf, size, bytes_read, NULL);
}

void reset_disk(HANDLE h)
{
    SetFilePointer(h, 0, 0, 0);
}

int32_t get_last_error()
{
    return GetLastError();
}

void close_disk(HANDLE h)
{
    CloseHandle(h);
}
