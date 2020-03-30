#include<stdio.h>
#include <pthread.h>
int arr[2];
void *inc(void* arg)
{
  arr[0]++;
  pthread_exit(NULL);
}

void *dec(void* arg)
{
  arr[1]--;
  pthread_exit(NULL);
}

int main(int argc, char **argv)
{
  pthread_t id1, id2;
  arr[0] = 3;
  arr[1] = 6;

  pthread_create(&id1, NULL, inc, NULL);
  pthread_create(&id2, NULL, dec, NULL);
  pthread_join(id1, 0);
  pthread_join(id2, 0);
  printf("Result: %d\n", arr[0]+arr[1]);
  return 0;
}
