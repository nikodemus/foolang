clang-format.exe -i host/generated_* && clang -o tmp_transpile_test.exe -fsanitize=address -fsanitize=undefined -fno-omit-frame-pointer -g -Wall --std=c17 host/main.c && ./tmp_transpile_test.exe
