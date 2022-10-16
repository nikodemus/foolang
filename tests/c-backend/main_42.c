/**
 * Foolang generated code - do not edit by hand!
 */

#include "foolang.h"

struct FooClass foo_class_1;
struct FooClass foo_class_2;
struct FooClass foo_class_3;
struct FooClass foo_class_4;
struct FooClass foo_class_5;
struct FooClass foo_class_6;
struct FooClass foo_class_7;

// "Boolean"
struct FooBytes foo_string_8 = {
    .size = 7,
    .data = { 'B','o','o','l','e','a','n', 0 }
};

// "ifFalse:"
struct FooBytes foo_string_11 = {
    .size = 8,
    .data = { 'i','f','F','a','l','s','e',':', 0 }
};

// #ifFalse:
struct FooSelector foo_selector_10 = {
    .name = &foo_string_11
};

// #<BuiltinMethodImpl ifFalse:>
char* foo_builtin_12(char* sp, struct Actor* actor) {
    (void)sp;
    (void)actor;
    printf("TODO: #<CpsSelector ifFalse:>");
    exit(1);
}

// #<BuiltinMethod Boolean#ifFalse:>
struct FooMethod foo_method_9 = {
    .home = &foo_class_1,
    .selector = &foo_selector_10,
    .method_function = foo_builtin_12
};

// "ifTrue:"
struct FooBytes foo_string_15 = {
    .size = 7,
    .data = { 'i','f','T','r','u','e',':', 0 }
};

// #ifTrue:
struct FooSelector foo_selector_14 = {
    .name = &foo_string_15
};

// #<BuiltinMethodImpl ifTrue:>
char* foo_builtin_16(char* sp, struct Actor* actor) {
    (void)sp;
    (void)actor;
    printf("TODO: #<CpsSelector ifTrue:>");
    exit(1);
}

// #<BuiltinMethod Boolean#ifTrue:>
struct FooMethod foo_method_13 = {
    .home = &foo_class_1,
    .selector = &foo_selector_14,
    .method_function = foo_builtin_16
};

// "ifTrue:ifFalse:"
struct FooBytes foo_string_19 = {
    .size = 15,
    .data = { 'i','f','T','r','u','e',':','i','f','F','a','l','s','e',':', 0 }
};

// #ifTrue:ifFalse:
struct FooSelector foo_selector_18 = {
    .name = &foo_string_19
};

// #<BuiltinMethodImpl ifTrue:ifFalse:>
char* foo_builtin_20(char* sp, struct Actor* actor) {
    (void)sp;
    (void)actor;
    printf("TODO: #<CpsSelector ifTrue:ifFalse:>");
    exit(1);
}

// #<BuiltinMethod Boolean#ifTrue:ifFalse:>
struct FooMethod foo_method_17 = {
    .home = &foo_class_1,
    .selector = &foo_selector_18,
    .method_function = foo_builtin_20
};

// "opaqueIdentity"
struct FooBytes foo_string_23 = {
    .size = 14,
    .data = { 'o','p','a','q','u','e','I','d','e','n','t','i','t','y', 0 }
};

// #opaqueIdentity
struct FooSelector foo_selector_22 = {
    .name = &foo_string_23
};

// #<BuiltinMethodImpl opaqueIdentity>
char* foo_builtin_24(char* sp, struct Actor* actor) {
    char* bp = actor->bp = sp - 4 * sizeof(datum_t);
    datum_t cont = READ_DATUM(bp, 0);
    datum_t class = READ_DATUM(bp, 1);
    datum_t datum = READ_DATUM(bp, 2);
    sp = bp;
    PUSH_DATUM(sp, class);
    PUSH_DATUM(sp, datum);
    PUSH_DATUM(sp, OBJS(1));
    PUSH_DATUM(sp, cont);
    return sp;
}

// #<BuiltinMethod Boolean#opaqueIdentity>
struct FooMethod foo_method_21 = {
    .home = &foo_class_1,
    .selector = &foo_selector_22,
    .method_function = foo_builtin_24
};

// #<MethodDictionary Boolean>
struct FooMethodDictionary foo_methods_25 = {
    .size = 4,
    .data = {
        &foo_method_9, // #<BuiltinMethod Boolean#ifFalse:>
        &foo_method_13, // #<BuiltinMethod Boolean#ifTrue:>
        &foo_method_17, // #<BuiltinMethod Boolean#ifTrue:ifFalse:>
        &foo_method_21, // #<BuiltinMethod Boolean#opaqueIdentity>
    }
};

// #<Class Boolean>
struct FooClass foo_class_1 = {
    .name = &foo_string_8,
    .own_class = &foo_class_2,
    .methods = &foo_methods_25
};

// "Boolean class"
struct FooBytes foo_string_26 = {
    .size = 13,
    .data = { 'B','o','o','l','e','a','n',' ','c','l','a','s','s', 0 }
};

// #<BuiltinMethod Boolean class#opaqueIdentity>
struct FooMethod foo_method_27 = {
    .home = &foo_class_2,
    .selector = &foo_selector_22,
    .method_function = foo_builtin_24
};

// "typecheck:"
struct FooBytes foo_string_30 = {
    .size = 10,
    .data = { 't','y','p','e','c','h','e','c','k',':', 0 }
};

// #typecheck:
struct FooSelector foo_selector_29 = {
    .name = &foo_string_30
};

// #<BuiltinMethodImpl typecheck:>
char* foo_builtin_31(char* sp, struct Actor* actor) {
    char* bp = actor->bp = sp - 6 * sizeof(datum_t);
    datum_t expectedClass = READ_DATUM(bp, 1);
    datum_t actualClass = READ_DATUM(bp, 3);
    if (expectedClass == actualClass) {
        datum_t cont = READ_DATUM(bp, 0);
        datum_t actualDatum = READ_DATUM(bp, 4);
        sp = bp;
        PUSH_DATUM(sp, actualClass);
        PUSH_DATUM(sp, actualDatum);
        PUSH_DATUM(sp, OBJS(1));
        PUSH_DATUM(sp, cont);
        return sp;
    } else {
        return runtime_type_error(sp, actor);
    }
}

// #<BuiltinMethod Boolean class#typecheck:>
struct FooMethod foo_method_28 = {
    .home = &foo_class_2,
    .selector = &foo_selector_29,
    .method_function = foo_builtin_31
};

// #<MethodDictionary Boolean class>
struct FooMethodDictionary foo_methods_32 = {
    .size = 2,
    .data = {
        &foo_method_27, // #<BuiltinMethod Boolean class#opaqueIdentity>
        &foo_method_28, // #<BuiltinMethod Boolean class#typecheck:>
    }
};

// #<Class Boolean class>
struct FooClass foo_class_2 = {
    .name = &foo_string_26,
    .own_class = &foo_class_3,
    .methods = &foo_methods_32
};

// "Class"
struct FooBytes foo_string_33 = {
    .size = 5,
    .data = { 'C','l','a','s','s', 0 }
};

// #<BuiltinMethod Class#opaqueIdentity>
struct FooMethod foo_method_34 = {
    .home = &foo_class_3,
    .selector = &foo_selector_22,
    .method_function = foo_builtin_24
};

// #<BuiltinMethodImpl typecheck:>
char* foo_builtin_36(char* sp, struct Actor* actor) {
    char* bp = actor->bp = sp - 6 * sizeof(datum_t);
    datum_t expectedClass = READ_DATUM(bp, 1);
    datum_t actualClass = READ_DATUM(bp, 3);
    if (expectedClass == actualClass) {
        datum_t cont = READ_DATUM(bp, 0);
        datum_t actualDatum = READ_DATUM(bp, 4);
        sp = bp;
        PUSH_DATUM(sp, actualClass);
        PUSH_DATUM(sp, actualDatum);
        PUSH_DATUM(sp, OBJS(1));
        PUSH_DATUM(sp, cont);
        return sp;
    } else {
        return runtime_type_error(sp, actor);
    }
}

// #<BuiltinMethod Class#typecheck:>
struct FooMethod foo_method_35 = {
    .home = &foo_class_3,
    .selector = &foo_selector_29,
    .method_function = foo_builtin_36
};

// #<MethodDictionary Class>
struct FooMethodDictionary foo_methods_37 = {
    .size = 2,
    .data = {
        &foo_method_34, // #<BuiltinMethod Class#opaqueIdentity>
        &foo_method_35, // #<BuiltinMethod Class#typecheck:>
    }
};

// #<Class>
struct FooClass foo_class_3 = {
    .name = &foo_string_33,
    .own_class = &foo_class_3,
    .methods = &foo_methods_37
};

// "Integer"
struct FooBytes foo_string_38 = {
    .size = 7,
    .data = { 'I','n','t','e','g','e','r', 0 }
};

// "+"
struct FooBytes foo_string_41 = {
    .size = 1,
    .data = { '+', 0 }
};

// #+
struct FooSelector foo_selector_40 = {
    .name = &foo_string_41
};

// #<BuiltinMethodImpl +>
char* foo_builtin_42(char* sp, struct Actor* actor) {
    (void)sp;
    (void)actor;
    printf("TODO: #<CpsSelector +>");
    exit(1);
}

// #<BuiltinMethod Integer#+>
struct FooMethod foo_method_39 = {
    .home = &foo_class_4,
    .selector = &foo_selector_40,
    .method_function = foo_builtin_42
};

// "addInteger:"
struct FooBytes foo_string_45 = {
    .size = 11,
    .data = { 'a','d','d','I','n','t','e','g','e','r',':', 0 }
};

// #addInteger:
struct FooSelector foo_selector_44 = {
    .name = &foo_string_45
};

// #<BuiltinMethodImpl addInteger:>
char* foo_builtin_46(char* sp, struct Actor* actor) {
    (void)sp;
    (void)actor;
    printf("TODO: #<CpsSelector addInteger:>");
    exit(1);
}

// #<BuiltinMethod Integer#addInteger:>
struct FooMethod foo_method_43 = {
    .home = &foo_class_4,
    .selector = &foo_selector_44,
    .method_function = foo_builtin_46
};

// #<BuiltinMethod Integer#opaqueIdentity>
struct FooMethod foo_method_47 = {
    .home = &foo_class_4,
    .selector = &foo_selector_22,
    .method_function = foo_builtin_24
};

// #<MethodDictionary Integer>
struct FooMethodDictionary foo_methods_48 = {
    .size = 3,
    .data = {
        &foo_method_39, // #<BuiltinMethod Integer#+>
        &foo_method_43, // #<BuiltinMethod Integer#addInteger:>
        &foo_method_47, // #<BuiltinMethod Integer#opaqueIdentity>
    }
};

// #<Class Integer>
struct FooClass foo_class_4 = {
    .name = &foo_string_38,
    .own_class = &foo_class_5,
    .methods = &foo_methods_48
};

// "Integer class"
struct FooBytes foo_string_49 = {
    .size = 13,
    .data = { 'I','n','t','e','g','e','r',' ','c','l','a','s','s', 0 }
};

// #<BuiltinMethod Integer class#opaqueIdentity>
struct FooMethod foo_method_50 = {
    .home = &foo_class_5,
    .selector = &foo_selector_22,
    .method_function = foo_builtin_24
};

// #<BuiltinMethodImpl typecheck:>
char* foo_builtin_52(char* sp, struct Actor* actor) {
    char* bp = actor->bp = sp - 6 * sizeof(datum_t);
    datum_t expectedClass = READ_DATUM(bp, 1);
    datum_t actualClass = READ_DATUM(bp, 3);
    if (expectedClass == actualClass) {
        datum_t cont = READ_DATUM(bp, 0);
        datum_t actualDatum = READ_DATUM(bp, 4);
        sp = bp;
        PUSH_DATUM(sp, actualClass);
        PUSH_DATUM(sp, actualDatum);
        PUSH_DATUM(sp, OBJS(1));
        PUSH_DATUM(sp, cont);
        return sp;
    } else {
        return runtime_type_error(sp, actor);
    }
}

// #<BuiltinMethod Integer class#typecheck:>
struct FooMethod foo_method_51 = {
    .home = &foo_class_5,
    .selector = &foo_selector_29,
    .method_function = foo_builtin_52
};

// #<MethodDictionary Integer class>
struct FooMethodDictionary foo_methods_53 = {
    .size = 2,
    .data = {
        &foo_method_50, // #<BuiltinMethod Integer class#opaqueIdentity>
        &foo_method_51, // #<BuiltinMethod Integer class#typecheck:>
    }
};

// #<Class Integer class>
struct FooClass foo_class_5 = {
    .name = &foo_string_49,
    .own_class = &foo_class_3,
    .methods = &foo_methods_53
};

// "Main"
struct FooBytes foo_string_54 = {
    .size = 4,
    .data = { 'M','a','i','n', 0 }
};

// #<BuiltinMethod Main#opaqueIdentity>
struct FooMethod foo_method_55 = {
    .home = &foo_class_6,
    .selector = &foo_selector_22,
    .method_function = foo_builtin_24
};

// "run:in:"
struct FooBytes foo_string_58 = {
    .size = 7,
    .data = { 'r','u','n',':','i','n',':', 0 }
};

// #run:in:
struct FooSelector foo_selector_57 = {
    .name = &foo_string_58
};

// #<CpsGraph Main#run:in:>
char* foo_method_function_59(char* sp, struct Actor* actor) {
    // 0 - $return
    // 1 - $receiver class
    // 2 - $receiver datum
    // 3 - command class
    // 4 - command datum
    // 5 - system class
    // 6 - system datum
    // 7 - layout word
    // -
    char* bp = actor->bp = sp - 8 * sizeof(datum_t);
    datum_t d0 = READ_DATUM(bp, 0);
    sp = bp;
    PUSH_DATUM(sp, &foo_class_4);
    PUSH_DATUM(sp, 42);
    PUSH_DATUM(sp, OBJS(1));
    PUSH_DATUM(sp, d0);
    return sp;
}

// #<UserMethod Main#run:in:>
struct FooMethod foo_method_56 = {
    .home = &foo_class_6,
    .selector = &foo_selector_57,
    .method_function = foo_method_function_59
};

// #<MethodDictionary Main>
struct FooMethodDictionary foo_methods_60 = {
    .size = 2,
    .data = {
        &foo_method_55, // #<BuiltinMethod Main#opaqueIdentity>
        &foo_method_56, // #<UserMethod Main#run:in:>
    }
};

// #<Class Main>
struct FooClass foo_class_6 = {
    .name = &foo_string_54,
    .own_class = &foo_class_7,
    .methods = &foo_methods_60
};

// "Main class"
struct FooBytes foo_string_61 = {
    .size = 10,
    .data = { 'M','a','i','n',' ','c','l','a','s','s', 0 }
};

// "new"
struct FooBytes foo_string_64 = {
    .size = 3,
    .data = { 'n','e','w', 0 }
};

// #new
struct FooSelector foo_selector_63 = {
    .name = &foo_string_64
};

// #<BuiltinMethodImpl new>
char* foo_builtin_65(char* sp, struct Actor* actor) {
    char* bp = actor->bp = sp - 4 * sizeof(datum_t);
    datum_t cont = READ_DATUM(bp, 0);
    sp = bp;
    PUSH_DATUM(sp, &foo_class_6);
    PUSH_DATUM(sp, 0); // only empty classes for now!
    PUSH_DATUM(sp, OBJS(1));
    PUSH_DATUM(sp, cont);
    return sp;
}

// #<BuiltinMethod Main class#new>
struct FooMethod foo_method_62 = {
    .home = &foo_class_7,
    .selector = &foo_selector_63,
    .method_function = foo_builtin_65
};

// #<BuiltinMethod Main class#opaqueIdentity>
struct FooMethod foo_method_66 = {
    .home = &foo_class_7,
    .selector = &foo_selector_22,
    .method_function = foo_builtin_24
};

// #<BuiltinMethodImpl typecheck:>
char* foo_builtin_68(char* sp, struct Actor* actor) {
    char* bp = actor->bp = sp - 6 * sizeof(datum_t);
    datum_t expectedClass = READ_DATUM(bp, 1);
    datum_t actualClass = READ_DATUM(bp, 3);
    if (expectedClass == actualClass) {
        datum_t cont = READ_DATUM(bp, 0);
        datum_t actualDatum = READ_DATUM(bp, 4);
        sp = bp;
        PUSH_DATUM(sp, actualClass);
        PUSH_DATUM(sp, actualDatum);
        PUSH_DATUM(sp, OBJS(1));
        PUSH_DATUM(sp, cont);
        return sp;
    } else {
        return runtime_type_error(sp, actor);
    }
}

// #<BuiltinMethod Main class#typecheck:>
struct FooMethod foo_method_67 = {
    .home = &foo_class_7,
    .selector = &foo_selector_29,
    .method_function = foo_builtin_68
};

// #<MethodDictionary Main class>
struct FooMethodDictionary foo_methods_69 = {
    .size = 3,
    .data = {
        &foo_method_62, // #<BuiltinMethod Main class#new>
        &foo_method_66, // #<BuiltinMethod Main class#opaqueIdentity>
        &foo_method_67, // #<BuiltinMethod Main class#typecheck:>
    }
};

// #<Class Main class>
struct FooClass foo_class_7 = {
    .name = &foo_string_61,
    .own_class = &foo_class_3,
    .methods = &foo_methods_69
};

char* main_actor_exit(char* sp, struct Actor* actor) {
    datum_t res = READ_DATUM(sp, -2);
    printf(" INFO - main actor exit: %d\n", (int)res);
    atomic_store(&actor->state, ActorExiting);
    actor->sp = sp;
    return 0;
}

char* main_actor_entry(char* sp, struct Actor* actor) {
    (void)actor;
    PUSH_DATUM(sp, main_actor_exit);
    PUSH_DATUM(sp, &foo_class_7); // Main class
    PUSH_DATUM(sp, &foo_class_6); // Main
    PUSH_DATUM(sp, 0); // FIXME Array
    PUSH_DATUM(sp, 0); // FIXME    command datum
    PUSH_DATUM(sp, 0); // FIXME System
    PUSH_DATUM(sp, 0); // FIXME    system datum
    PUSH_DATUM(sp, IMMS(1) | OBJS(3));
    PUSH_DATUM(sp, foo_method_function_59);
    return sp;
}

int main(int argc, char** argv) {
    (void)argc;
    (void)argv;
    struct ExecutorPool* pool = make_ExecutorPool(1);
    struct Actor* mainActor = make_Actor(main_actor_entry, NULL);
    enqueue_actor(pool->executors[0]->queue, mainActor);
    start_pool(pool);
    wait_for_actor_exit(mainActor, 0);
    free_Actor(mainActor);
    stop_pool(pool);
    free_ExecutorPool(pool);
    printf(" INFO - exit\n");
    return 0;
}
