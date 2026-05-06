/**
 * @file src/platform/windows/nvprefs/driver_settings.h
 * @brief Declarations for nvidia driver settings.
 */
#pragma once

// nvapi headers
// disable clang-format header reordering
// as <NvApiDriverSettings.h> needs types from <nvapi.h>
// clang-format off
// Undefine MinGW SAL macros so nvapi_lite_salstart.h can correctly redefine them to empty
#ifdef __GNUC__
#undef __in
#undef __in_bcount
#undef __in_ecount
#undef __in_opt
#undef __in_ecount_opt
#undef __out
#undef __out_bcount
#undef __out_ecount
#undef __out_opt
#undef __out_ecount_full_opt
#undef __out_ecount_part_opt
#undef __inout
#undef __inout_bcount
#undef __inout_ecount
#undef __inout_opt
#undef __inout_ecount_part_opt
#undef __inout_ecount_full
#undef __inout_ecount_full_opt
#endif

#include <nvapi.h>
#include <NvApiDriverSettings.h>
// clang-format on

// local includes
#include "undo_data.h"

namespace nvprefs {

  class driver_settings_t {
  public:
    ~driver_settings_t();

    bool
    init();

    void
    destroy();

    bool
    load_settings();

    bool
    save_settings();

    bool
    restore_global_profile_to_undo(const undo_data_t &undo_data);

    bool
    check_and_modify_global_profile(std::optional<undo_data_t> &undo_data);

    bool
    check_and_modify_application_profile(bool &modified);

  private:
    NvDRSSessionHandle session_handle = 0;
  };

}  // namespace nvprefs
