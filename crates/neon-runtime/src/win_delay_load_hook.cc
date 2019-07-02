/*
 * When this file is linked to a DLL, it sets up a delay-load hook that
 * intervenes when the DLL is trying to load 'node.exe' or 'iojs.exe'
 * dynamically. Instead of trying to locate the .exe file it'll just return
 * a handle to the process image.
 *
 * This allows compiled addons to work when node.exe or iojs.exe is renamed.
 */

 
#include <windows.h>

#include <delayimp.h>
#include <string.h>

static bool module_name_checked = false;
static bool ignore_delay_load = false;

static bool check_module_name() {
  const DWORD len = 512;
  wchar_t module_name[len] = {0};
  auto actual_len = GetModuleFileNameW(nullptr, module_name, len);
  if (len == actual_len) {
    return false;
  }

  wchar_t drive[_MAX_DRIVE] = {0};
  wchar_t dir[_MAX_DIR] = {0};
  wchar_t fname[_MAX_FNAME] = {0};
  wchar_t ext[_MAX_EXT] = {0};
  if (_wsplitpath_s(module_name, drive, _MAX_DRIVE, dir, _MAX_DIR, fname, _MAX_FNAME, ext, _MAX_EXT) != 0) {
    return false;
  }

  return _wcsicmp(ext, L"node") == 0;
}

FARPROC WINAPI load_exe_hook(unsigned int event, DelayLoadInfo* info) {
  HMODULE m;
  if (event != dliNotePreLoadLibrary)
    return NULL;

  if (!module_name_checked) {
    ignore_delay_load = check_module_name();
    module_name_checked = true;
  }

  if (!ignore_delay_load && stricmp(info->szDll, "node.exe") != 0)
    return NULL;

	
  m = GetModuleHandle(NULL);
  return (FARPROC) m;
}

decltype(__pfnDliNotifyHook2) __pfnDliNotifyHook2 = load_exe_hook;
