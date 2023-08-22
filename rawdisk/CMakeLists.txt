cmake_minimum_required(VERSION 3.9)
project(rawdisk VERSION 1.0.1 DESCRIPTION "raw disk load library")
include(GNUInstallDirs)
add_library(rawdisk STATIC src/disk.c)
set_target_properties(rawdisk PROPERTIES
    VERSION ${PROJECT_VERSION}
    SOVERSION 1
    PUBLIC_HEADER api/rawdisk.h)
configure_file(rawdisk.pc.in rawdisk.pc @ONLY)
target_include_directories(rawdisk PRIVATE .)
install(TARGETS rawdisk
    LIBRARY DESTINATION ${CMAKE_INSTALL_LIBDIR}
    PUBLIC_HEADER DESTINATION ${CMAKE_INSTALL_INCLUDEDIR})
install(FILES ${CMAKE_BINARY_DIR}/rawdisk.pc
    DESTINATION ${CMAKE_INSTALL_DATAROOTDIR}/pkgconfig)
