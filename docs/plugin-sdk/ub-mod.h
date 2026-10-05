/* Unibrowser compiled mod C ABI (draft v1). */
#ifndef UB_MOD_H
#define UB_MOD_H

#include <stdint.h>

#define UB_MOD_LOADER_ABI 1u

typedef struct UbModHostV1 {
    uint32_t abi_version;
    void *host_ctx;
    /* Returns 0 on success. id and title are UTF-8, NUL-terminated. */
    int32_t (*register_command)(void *host_ctx, const char *id, const char *title);
} UbModHostV1;

#ifdef __cplusplus
extern "C" {
#endif

uint32_t ub_mod_loader_abi_version(void);
const char *ub_mod_api_version(void);
const char *ub_mod_info_json(void);
/* Returns 0 on success. */
int32_t ub_mod_register(const UbModHostV1 *host);

#ifdef __cplusplus
}
#endif

#endif /* UB_MOD_H */
