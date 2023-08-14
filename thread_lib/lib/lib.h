#ifndef _LIB_H

void hitsz_thread_init();
// 获取当前线程对应编号
int hitsz_thread_self();
int hitsz_thread_create();
void hitsz_thread_join();

#endif // !_LIB_H
