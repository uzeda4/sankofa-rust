# Local build: libdav1d.a is prebuilt with meson (see native/image-decoder/build.ps1) and found through
# DAV1D_PREBUILT_DIR (the meson build dir) and DAV1D_SOURCE_DIR.
set(DAV1D_FILENAME "${DAV1D_PREBUILT_DIR}/src/libdav1d.a")
if(NOT EXISTS "${DAV1D_FILENAME}")
  message(FATAL_ERROR "libdav1d: ${DAV1D_FILENAME} is missing")
endif()
set(DAV1D_FOUND 1)
set(DAV1D_LIBRARIES "${DAV1D_FILENAME}")
set(DAV1D_INCLUDE_DIR
  "${DAV1D_PREBUILT_DIR}"
  "${DAV1D_PREBUILT_DIR}/include"
  "${DAV1D_PREBUILT_DIR}/include/dav1d"
  "${DAV1D_SOURCE_DIR}/include"
)
