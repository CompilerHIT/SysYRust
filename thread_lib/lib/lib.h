



extern long *tmp_mem;
extern long *tids;
extern int num_tid;

void* get_next_pc();
void thread_init();
//��ȡ��ǰ�̶߳�Ӧ���
int thread_self();
int thread_create();
void thread_join();