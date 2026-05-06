#!/bin/bash
export PATH="/usr/bin:$PATH"
set -e

echo "=== Applying GCC Fix for NVAPI SDK ==="

# 1. Create the fix header if it doesn't exist
cat <<EOF > third-party/nvapi-open-source-sdk/nvapi_gcc_fix.h
#ifndef NVAPI_GCC_FIX_H
#define NVAPI_GCC_FIX_H

#ifdef __GNUC__
    #include <_mingw.h>
    
    // Undefine all SAL macros that MinGW might have defined
    #undef __in
    #undef __out
    #undef __inout
    #undef __in_opt
    #undef __out_opt
    #undef __inout_opt
    #undef __in_bcount
    #undef __out_bcount
    #undef __in_ecount
    #undef __out_ecount
    #undef __in_ecount_opt
    #undef __out_ecount_opt
    #undef __in_bcount_opt
    #undef __out_bcount_opt
    #undef __deref_out
    #undef __deref_out_opt
    #undef __success
    #undef __checkReturn
    #undef __field_ecount
    #undef __field_bcount
    #undef __callback
    #undef __format_string
    #undef __reserved
    
    // Define them to nothing
    #define __in
    #define __out
    #define __inout
    #define __in_opt
    #define __out_opt
    #define __inout_opt
    #define __in_bcount(x)
    #define __out_bcount(x)
    #define __in_ecount(x)
    #define __out_ecount(x)
    #define __in_ecount_opt(x)
    #define __out_ecount_opt(x)
    #define __in_bcount_opt(x)
    #define __out_bcount_opt(x)
    #define __deref_out
    #define __deref_out_opt
    #define __success(x)
    #define __checkReturn
    #define __field_ecount(x)
    #define __field_bcount(x)
    #define __callback
    #define __format_string
    #define __reserved
    
    // Handle parameterized ones with a generic macro
    #define __in_ecount_full(x)
    #define __in_ecount_part(x,y)
    #define __out_ecount_full(x)
    #define __out_ecount_part(x,y)
    #define __inout_ecount_full(x)
    #define __inout_ecount_part(x,y)
    
#endif

#endif // NVAPI_GCC_FIX_H
EOF

# 2. Inject include into main headers if not already present
if ! grep -q "nvapi_gcc_fix.h" third-party/nvapi-open-source-sdk/nvapi.h; then
    sed -i '1i #include "nvapi_gcc_fix.h"' third-party/nvapi-open-source-sdk/nvapi.h
fi

if ! grep -q "nvapi_gcc_fix.h" third-party/nvapi-open-source-sdk/nvapi_interface.h; then
    sed -i '1i #include "nvapi_gcc_fix.h"' third-party/nvapi-open-source-sdk/nvapi_interface.h
fi

echo "=== GCC Fix Applied Successfully ==="
