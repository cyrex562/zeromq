// #if defined _WIN32_WCE

//#include "..\..\include\zmq.h"
// #include "..\..\src\err.hpp"

errno: i32;
int _doserrno;
int _sys_nerr;

char* error_desc_buff = null_mut();

char* strerror(errno: i32)
{
    if (null_mut() != error_desc_buff)
    {
        LocalFree(error_desc_buff);
        error_desc_buff = null_mut();
    }

    FormatMessage(
        FORMAT_MESSAGE_FROM_SYSTEM | FORMAT_MESSAGE_IGNORE_INSERTS | FORMAT_MESSAGE_ALLOCATE_BUFFER,
        null_mut(),
        errno,
        0,
        (LPTSTR)&error_desc_buff,
        1024,
        null_mut()
        );
    return error_desc_buff;
}

// #endif
