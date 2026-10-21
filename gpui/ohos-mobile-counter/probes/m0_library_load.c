// M0 device preflight only. This does not validate an app/HAP linker namespace.
#include <dlfcn.h>
#include <stdio.h>

struct SymbolProbe {
  const char *library;
  const char *symbol;
};

int main(void) {
  static const struct SymbolProbe probes[] = {
      {"libnative_drawing.so", "OH_Drawing_GpuContextCreate"},
      {"libnative_drawing.so", "OH_Drawing_SurfaceCreateOnScreen"},
      {"libnative_drawing.so", "OH_Drawing_SurfaceFlush"},
      {"libace_ndk.z.so", "OH_NativeXComponent_RegisterCallback"},
      {"libace_napi.z.so", "napi_module_register"},
  };
  int failures = 0;
  for (unsigned i = 0; i < sizeof(probes) / sizeof(probes[0]); ++i) {
    dlerror();
    void *library = dlopen(probes[i].library, RTLD_NOW | RTLD_LOCAL);
    if (library == NULL) {
      printf("LOAD_FAIL %s: %s\n", probes[i].library, dlerror());
      ++failures;
      continue;
    }
    dlerror();
    void *symbol = dlsym(library, probes[i].symbol);
    const char *error = dlerror();
    if (error != NULL || symbol == NULL) {
      printf("SYMBOL_FAIL %s %s: %s\n", probes[i].library, probes[i].symbol,
             error != NULL ? error : "null address");
      ++failures;
    } else {
      printf("OK %s %s\n", probes[i].library, probes[i].symbol);
    }
    dlclose(library);
  }
  return failures == 0 ? 0 : 1;
}
