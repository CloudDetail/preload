#define _GNU_SOURCE
#include <unistd.h>
#include <dlfcn.h>
#include <stdio.h>
#include <syslog.h>
#include <sys/syscall.h>
typedef int (*execve_func)(const char *filename, char *const argv[], char *const envp[]);
execve_func original_system_execve = NULL;

typedef int (*extra_execve_func)(const char *filename, char *const argv[], char *const envp[], execve_func execve_shim);
extra_execve_func apo_execve_func = NULL;

void init_execve_apo(void) __attribute__((constructor));
int system_execve_shim(const char *filename, char *const argv[], char *const envp[]);

void init_execve_apo(void)
{
    original_system_execve = dlsym(RTLD_NEXT, "execve");
    const char *error = dlerror();
    if (error != NULL || original_system_execve == execve)
    {
        original_system_execve = NULL;
    }

    char *instrument_lib_path = "/etc/apo/instrument/libapoinstrument.so";
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