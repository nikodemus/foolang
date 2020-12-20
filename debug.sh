set -euo pipefail
clang-format -i host/generated_*
clang \
    -o tmp_transpile_test.exe \
    -fsanitize=address -fsanitize=undefined \
    -fno-omit-frame-pointer -g -Wall --std=c11 \
    host/main.c host/system_windows.c
./tmp_transpile_test.exe
