#include <assert.h>
#include <stdint.h>
#include <sys/shm.h>

int32_t shmutils_create(uint32_t size) {
  static_assert(sizeof(int) == 4);

  return shmget(IPC_PRIVATE,
                size,
                IPC_CREAT | 0777);
}

uint8_t *shmutils_get_ptr(int32_t shmid) {
  static_assert(sizeof(char) == 1);

  return shmat(shmid, 0, 0);
}

void shmutils_free_remove(int32_t shmid, uint8_t *shmaddr) {
  shmdt(shmaddr);
  shmctl(shmid, IPC_RMID, 0);
}
