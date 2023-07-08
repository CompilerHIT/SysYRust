account=$1
local_path=$2
remote_path=$3

# 首先解除占用,如果发生了占用的话
# fusermount -u $local_path

#然后重新占用
sshfs $account:$remote_path  $local_path

