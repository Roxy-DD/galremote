/*********************************************************************************************************
|*                                                                                                        *|
|* SPDX-FileCopyrightText: Copyright (c) 2019-2024 NVIDIA CORPORATION & AFFILIATES. All rights reserved.  *|
|* SPDX-License-Identifier: MIT                                                                           *|
|*                                                                                                        *|
|* Permission is hereby granted, free of charge, to any person obtaining a                                *|
|* copy of this software and associated documentation files (the "Software"),                             *|
|* to deal in the Software without restriction, including without limitation                              *|
|* the rights to use, copy, modify, merge, publish, distribute, sublicense,                               *|
|* and/or sell copies of the Software, and to permit persons to whom the                                  *|
|* Software is furnished to do so, subject to the following conditions:                                   *|
|*                                                                                                        *|
|* The above copyright notice and this permission notice shall be included in                             *|
|* all copies or substantial portions of the Software.                                                    *|
|*                                                                                                        *|
|* THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND, EXPRESS OR                             *|
|* IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY,                               *|
|* FITNESS FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT.  IN NO EVENT SHALL                               *|
|* THE AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY CLAIM, DAMAGES OR OTHER                             *|
|* LIABILITY, WHETHER IN AN ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING                                *|
|* FROM, OUT OF OR IN CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER                                    *|
|* DEALINGS IN THE SOFTWARE.                                                                              *|
|*                                                                                                        *|
|*                                                                                                        *|
\*********************************************************************************************************/

#pragma once

#if defined(__GNUC__) || defined(__clang__)

// Force UNDEF all SAL annotations for GCC/Clang to clean up global namespace
#undef __ecount
#undef __bcount
#undef __in
#undef __in_ecount
#undef __in_bcount
#undef __in_z
#undef __in_ecount_z
#undef __in_bcount_z
#undef __in_nz
#undef __in_ecount_nz
#undef __in_bcount_nz
#undef __out
#undef __out_ecount
#undef __out_bcount
#undef __out_ecount_part
#undef __out_bcount_part
#undef __out_ecount_full
#undef __out_bcount_full
#undef __out_z
#undef __out_z_opt
#undef __out_ecount_z
#undef __out_bcount_z
#undef __out_ecount_part_z
#undef __out_bcount_part_z
#undef __out_ecount_full_z
#undef __out_bcount_full_z
#undef __out_nz
#undef __out_nz_opt
#undef __out_ecount_nz
#undef __out_bcount_nz
#undef __inout
#undef __inout_ecount
#undef __inout_bcount
#undef __inout_ecount_part
#undef __inout_bcount_part
#undef __inout_ecount_full
#undef __inout_bcount_full
#undef __inout_z
#undef __inout_ecount_z
#undef __inout_bcount_z
#undef __inout_nz
#undef __inout_ecount_nz
#undef __inout_bcount_nz
#undef __ecount_opt
#undef __bcount_opt
#undef __in_opt
#undef __in_ecount_opt
#undef __in_bcount_opt
#undef __in_z_opt
#undef __in_ecount_z_opt
#undef __in_bcount_z_opt
#undef __in_nz_opt
#undef __in_ecount_nz_opt
#undef __in_bcount_nz_opt
#undef __out_opt
#undef __out_ecount_opt
#undef __out_bcount_opt
#undef __out_ecount_part_opt
#undef __out_bcount_part_opt
#undef __out_ecount_full_opt
#undef __out_bcount_full_opt
#undef __out_ecount_z_opt
#undef __out_bcount_z_opt
#undef __out_ecount_part_z_opt
#undef __out_bcount_part_z_opt
#undef __out_ecount_full_z_opt
#undef __out_bcount_full_z_opt
#undef __out_ecount_nz_opt
#undef __out_bcount_nz_opt
#undef __inout_opt
#undef __inout_ecount_opt
#undef __inout_bcount_opt
#undef __inout_ecount_part_opt
#undef __inout_bcount_part_opt
#undef __inout_ecount_full_opt
#undef __inout_bcount_full_opt
#undef __inout_z_opt
#undef __inout_ecount_z_opt
#undef __inout_ecount_z_opt // duplicate
#undef __inout_bcount_z_opt
#undef __inout_nz_opt
#undef __inout_ecount_nz_opt
#undef __inout_bcount_nz_opt

#undef __deref_ecount
#undef __deref_bcount
#undef __deref_out
#undef __deref_out_ecount
#undef __deref_out_bcount
#undef __deref_out_ecount_part
#undef __deref_out_bcount_part
#undef __deref_out_ecount_full
#undef __deref_out_bcount_full
#undef __deref_out_z
#undef __deref_out_ecount_z
#undef __deref_out_bcount_z
#undef __deref_out_nz
#undef __deref_out_ecount_nz
#undef __deref_out_bcount_nz
#undef __deref_inout
#undef __deref_inout_z
#undef __deref_inout_ecount
#undef __deref_inout_bcount
#undef __deref_inout_ecount_part
#undef __deref_inout_bcount_part
#undef __deref_inout_ecount_full
#undef __deref_inout_bcount_full
#undef __deref_inout_z
#undef __deref_inout_ecount_z
#undef __deref_inout_bcount_z
#undef __deref_inout_nz
#undef __deref_inout_ecount_nz
#undef __deref_inout_bcount_nz
#undef __deref_ecount_opt
#undef __deref_bcount_opt
#undef __deref_out_opt
#undef __deref_out_ecount_opt
#undef __deref_out_bcount_opt
#undef __deref_out_ecount_part_opt
#undef __deref_out_bcount_part_opt
#undef __deref_out_ecount_full_opt
#undef __deref_out_bcount_full_opt
#undef __deref_out_z_opt
#undef __deref_out_ecount_z_opt
#undef __deref_out_bcount_z_opt
#undef __deref_out_nz_opt
#undef __deref_out_ecount_nz_opt
#undef __deref_out_bcount_nz_opt
#undef __deref_inout_opt
#undef __deref_inout_ecount_opt
#undef __deref_inout_bcount_opt
#undef __deref_inout_ecount_part_opt
#undef __deref_inout_bcount_part_opt
#undef __deref_inout_ecount_full_opt
#undef __deref_inout_bcount_full_opt
#undef __deref_inout_z_opt
#undef __deref_inout_ecount_z_opt
#undef __deref_inout_bcount_z_opt
#undef __deref_inout_nz_opt
#undef __deref_inout_ecount_nz_opt
#undef __deref_inout_bcount_nz_opt
#undef __deref_opt_ecount
#undef __deref_opt_bcount
#undef __deref_opt_out
#undef __deref_opt_out_z
#undef __deref_opt_out_ecount
#undef __deref_opt_out_bcount
#undef __deref_opt_out_ecount_part
#undef __deref_opt_out_bcount_part
#undef __deref_opt_out_ecount_full
#undef __deref_opt_out_bcount_full
#undef __deref_opt_inout
#undef __deref_opt_inout_ecount
#undef __deref_opt_inout_bcount
#undef __deref_opt_inout_ecount_part
#undef __deref_opt_inout_bcount_part
#undef __deref_opt_inout_ecount_full
#undef __deref_opt_inout_bcount_full
#undef __deref_opt_inout_z
#undef __deref_opt_inout_ecount_z
#undef __deref_opt_inout_bcount_z
#undef __deref_opt_inout_nz
#undef __deref_opt_inout_ecount_nz
#undef __deref_opt_inout_bcount_nz
#undef __deref_opt_ecount_opt
#undef __deref_opt_bcount_opt
#undef __deref_opt_out_opt
#undef __deref_opt_out_ecount_opt
#undef __deref_opt_out_bcount_opt
#undef __deref_opt_out_ecount_part_opt
#undef __deref_opt_out_bcount_part_opt
#undef __deref_opt_out_ecount_full_opt
#undef __deref_opt_out_bcount_full_opt
#undef __deref_opt_out_z_opt
#undef __deref_opt_out_ecount_z_opt
#undef __deref_opt_out_bcount_z_opt
#undef __deref_opt_out_nz_opt
#undef __deref_opt_out_ecount_nz_opt
#undef __deref_opt_out_bcount_nz_opt
#undef __deref_opt_inout_opt
#undef __deref_opt_inout_ecount_opt
#undef __deref_opt_inout_bcount_opt
#undef __deref_opt_inout_ecount_part_opt
#undef __deref_opt_inout_bcount_part_opt
#undef __deref_opt_inout_ecount_full_opt
#undef __deref_opt_inout_bcount_full_opt
#undef __deref_opt_inout_z_opt
#undef __deref_opt_inout_ecount_z_opt
#undef __deref_opt_inout_bcount_z_opt
#undef __deref_opt_inout_nz_opt
#undef __deref_opt_inout_ecount_nz_opt
#undef __deref_opt_inout_bcount_nz_opt

#undef __success
#undef _Ret_notnull_
#undef _Post_writable_byte_size_
#undef _Outptr_ 

// NOTE: We do NOT undefine NVAPI_INTERFACE here, as it is needed by the user.

#endif // __GNUC__ || __clang__
