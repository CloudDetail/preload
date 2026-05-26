#define _GNU_SOURCE
#include <unistd.h>
#include <dlfcn.h>
#include <stdio.h>
#include <string.h>
#include <sys/syscall.h>

#ifndef SYS_execve
#define SYS_execve 59
#endif

typedef int (*execve_func)(const char *filename, char *const argv[], char *const envp[]);
execve_func original_system_execve = NULL;

typedef int (*extra_execve_func)(const char *filename, char *const argv[], char *const envp[], execve_func execve_shim);
extra_execve_func apo_execve_func = NULL;

typedef void (*constructor_instrument_func)(char *const argv[], char *const envp[]);

int system_execve_shim(const char *filename, char *const argv[], char *const envp[]);

static int is_musl(void) {
    FILE *f = fopen("/proc/self/maps", "r");
    if (!f) return 0;
    char line[512];
    while (fgets(line, sizeof(line), f)) {
        if (strstr(line, "ld-musl")) {
            fclose(f);
            return 1;
        }
    }
    fclose(f);
    return 0;
}

static int current_process_may_need_instrument(void) {
    FILE *f = fopen("/proc/self/cmdline", "r");
    if (!f) return 0;
    char cmdline[512];
    size_t n = fread(cmdline, 1, sizeof(cmdline) - 1, f);
    fclose(f);
    if (n == 0) {
        return 0;
    }
    for (size_t i = 0; i < n; i++) {
        if (cmdline[i] == '\0') {
            cmdline[i] = ' ';
        }
    }
    cmdline[n] = '\0';
    return strstr(cmdline, "java") != NULL
        || strstr(cmdline, "python") != NULL
        || strstr(cmdline, "node") != NULL;
}

void init_execve_apo(int argc, char **argv, char **envp) __attribute__((constructor));
void init_execve_apo(int argc, char **argv, char **envp)
{
    (void)argc;
    char *instrument_lib_path;
    if (is_musl()) {
        instrument_lib_path = "/etc/apo/instrument/libapoinstrument_musl.so";
    } else {
        original_system_execve = dlsym(RTLD_NEXT, "execve");
        const char *error = dlerror();
        if (error != NULL || original_system_execve == execve)
        {
            original_system_execve = NULL;
        }
        instrument_lib_path = "/etc/apo/instrument/libapoinstrument.so";
    }
    void *handle = dlopen(instrument_lib_path, RTLD_NOW | RTLD_NODELETE);
    const char *error2 = dlerror();
    if (error2 != NULL || handle == NULL)
    {
        apo_execve_func = NULL;
        return;
    }
    apo_execve_func = dlsym(handle, "apo_execve");
    const char *error3 = dlerror();
    if (error3 != NULL || apo_execve_func == NULL)
    {
        apo_execve_func = NULL;
    }
    if (current_process_may_need_instrument())
    {
        constructor_instrument_func constructor_instrument = dlsym(handle, "apo_instrument_current_process");
        const char *error4 = dlerror();
        if (error4 == NULL && constructor_instrument != NULL)
        {
            constructor_instrument(argv, envp);
        }
    }
    dlclose(handle);
}

int execve(const char *filename, char *const argv[], char *const envp[])
{
    int res;
    if (apo_execve_func == NULL)
    {
        res = system_execve_shim(filename, argv, envp);
    }
    else
    {
        res = apo_execve_func(filename, argv, envp, &system_execve_shim);
    }
    return res;
}

int system_execve_shim(const char *filename, char *const argv[], char *const envp[])
{
    int res;
    if (original_system_execve == NULL)
    {
        res = syscall(SYS_execve, filename, argv, envp);
    }
    else
    {
        res = original_system_execve(filename, argv, envp);
    }
    return res;
}
