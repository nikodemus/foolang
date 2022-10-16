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

// "new"
struct FooBytes foo_string_29 = {
    .size = 3,
    .data = { 'n','e','w', 0 }
};

// #new
struct FooSelector foo_selector_28 = {
    .name = &foo_string_29
};

// #<BuiltinMethodImpl new>
char* foo_builtin_30(char* sp, struct Actor* actor) {
    char* bp = actor->bp = sp - 4 * sizeof(datum_t);
    datum_t cont = READ_DATUM(bp, 0);
    sp = bp;
    PUSH_DATUM(sp, &foo_class_1);
    PUSH_DATUM(sp, 0); // only empty classes for now!
    PUSH_DATUM(sp, OBJS(1));
    PUSH_DATUM(sp, cont);
    return sp;
}

// #<BuiltinMethod Boolean class#new>
struct FooMethod foo_method_27 = {
    .home = &foo_class_2,
    .selector = &foo_selector_28,
    .method_function = foo_builtin_30
};

// #<BuiltinMethod Boolean class#opaqueIdentity>
struct FooMethod foo_method_31 = {
    .home = &foo_class_2,
    .selector = &foo_selector_22,
    .method_function = foo_builtin_24
};

// "typecheck:"
struct FooBytes foo_string_34 = {
    .size = 10,
    .data = { 't','y','p','e','c','h','e','c','k',':', 0 }
};

// #typecheck:
struct FooSelector foo_selector_33 = {
    .name = &foo_string_34
};

// #<BuiltinMethodImpl typecheck:>
char* foo_builtin_35(char* sp, struct Actor* actor) {
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
struct FooMethod foo_method_32 = {
    .home = &foo_class_2,
    .selector = &foo_selector_33,
    .method_function = foo_builtin_35
};

// #<MethodDictionary Boolean class>
struct FooMethodDictionary foo_methods_36 = {
    .size = 3,
    .data = {
        &foo_method_27, // #<BuiltinMethod Boolean class#new>
        &foo_method_31, // #<BuiltinMethod Boolean class#opaqueIdentity>
        &foo_method_32, // #<BuiltinMethod Boolean class#typecheck:>
    }
};

// #<Class Boolean class>
struct FooClass foo_class_2 = {
    .name = &foo_string_26,
    .own_class = &foo_class_3,
    .methods = &foo_methods_36
};

// "Class"
struct FooBytes foo_string_37 = {
    .size = 5,
    .data = { 'C','l','a','s','s', 0 }
};

// #<BuiltinMethodImpl new>
char* foo_builtin_39(char* sp, struct Actor* actor) {
    char* bp = actor->bp = sp - 4 * sizeof(datum_t);
    datum_t cont = READ_DATUM(bp, 0);
    sp = bp;
    PUSH_DATUM(sp, &foo_class_3);
    PUSH_DATUM(sp, 0); // only empty classes for now!
    PUSH_DATUM(sp, OBJS(1));
    PUSH_DATUM(sp, cont);
    return sp;
}

// #<BuiltinMethod Class#new>
struct FooMethod foo_method_38 = {
    .home = &foo_class_3,
    .selector = &foo_selector_28,
    .method_function = foo_builtin_39
};

// #<BuiltinMethod Class#opaqueIdentity>
struct FooMethod foo_method_40 = {
    .home = &foo_class_3,
    .selector = &foo_selector_22,
    .method_function = foo_builtin_24
};

// #<BuiltinMethodImpl typecheck:>
char* foo_builtin_42(char* sp, struct Actor* actor) {
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
struct FooMethod foo_method_41 = {
    .home = &foo_class_3,
    .selector = &foo_selector_33,
    .method_function = foo_builtin_42
};

// #<MethodDictionary Class>
struct FooMethodDictionary foo_methods_43 = {
    .size = 3,
    .data = {
        &foo_method_38, // #<BuiltinMethod Class#new>
        &foo_method_40, // #<BuiltinMethod Class#opaqueIdentity>
        &foo_method_41, // #<BuiltinMethod Class#typecheck:>
    }
};

// #<Class>
struct FooClass foo_class_3 = {
    .name = &foo_string_37,
    .own_class = &foo_class_3,
    .methods = &foo_methods_43
};

// "Integer"
struct FooBytes foo_string_44 = {
    .size = 7,
    .data = { 'I','n','t','e','g','e','r', 0 }
};

// "+"
struct FooBytes foo_string_47 = {
    .size = 1,
    .data = { '+', 0 }
};

// #+
struct FooSelector foo_selector_46 = {
    .name = &foo_string_47
};

// #<BuiltinMethodImpl +>
char* foo_builtin_48(char* sp, struct Actor* actor) {
    (void)sp;
    (void)actor;
    printf("TODO: #<CpsSelector +>");
    exit(1);
}

// #<BuiltinMethod Integer#+>
struct FooMethod foo_method_45 = {
    .home = &foo_class_4,
    .selector = &foo_selector_46,
    .method_function = foo_builtin_48
};

// "addInteger:"
struct FooBytes foo_string_51 = {
    .size = 11,
    .data = { 'a','d','d','I','n','t','e','g','e','r',':', 0 }
};

// #addInteger:
struct FooSelector foo_selector_50 = {
    .name = &foo_string_51
};

// #<BuiltinMethodImpl addInteger:>
char* foo_builtin_52(char* sp, struct Actor* actor) {
    (void)sp;
    (void)actor;
    printf("TODO: #<CpsSelector addInteger:>");
    exit(1);
}

// #<BuiltinMethod Integer#addInteger:>
struct FooMethod foo_method_49 = {
    .home = &foo_class_4,
    .selector = &foo_selector_50,
    .method_function = foo_builtin_52
};

// #<BuiltinMethod Integer#opaqueIdentity>
struct FooMethod foo_method_53 = {
    .home = &foo_class_4,
    .selector = &foo_selector_22,
    .method_function = foo_builtin_24
};

// #<MethodDictionary Integer>
struct FooMethodDictionary foo_methods_54 = {
    .size = 3,
    .data = {
        &foo_method_45, // #<BuiltinMethod Integer#+>
        &foo_method_49, // #<BuiltinMethod Integer#addInteger:>
        &foo_method_53, // #<BuiltinMethod Integer#opaqueIdentity>
    }
};

// #<Class Integer>
struct FooClass foo_class_4 = {
    .name = &foo_string_44,
    .own_class = &foo_class_5,
    .methods = &foo_methods_54
};

// "Integer class"
struct FooBytes foo_string_55 = {
    .size = 13,
    .data = { 'I','n','t','e','g','e','r',' ','c','l','a','s','s', 0 }
};

// #<BuiltinMethodImpl new>
char* foo_builtin_57(char* sp, struct Actor* actor) {
    char* bp = actor->bp = sp - 4 * sizeof(datum_t);
    datum_t cont = READ_DATUM(bp, 0);
    sp = bp;
    PUSH_DATUM(sp, &foo_class_4);
    PUSH_DATUM(sp, 0); // only empty classes for now!
    PUSH_DATUM(sp, OBJS(1));
    PUSH_DATUM(sp, cont);
    return sp;
}

// #<BuiltinMethod Integer class#new>
struct FooMethod foo_method_56 = {
    .home = &foo_class_5,
    .selector = &foo_selector_28,
    .method_function = foo_builtin_57
};

// #<BuiltinMethod Integer class#opaqueIdentity>
struct FooMethod foo_method_58 = {
    .home = &foo_class_5,
    .selector = &foo_selector_22,
    .method_function = foo_builtin_24
};

// #<BuiltinMethodImpl typecheck:>
char* foo_builtin_60(char* sp, struct Actor* actor) {
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
struct FooMethod foo_method_59 = {
    .home = &foo_class_5,
    .selector = &foo_selector_33,
    .method_function = foo_builtin_60
};

// #<MethodDictionary Integer class>
struct FooMethodDictionary foo_methods_61 = {
    .size = 3,
    .data = {
        &foo_method_56, // #<BuiltinMethod Integer class#new>
        &foo_method_58, // #<BuiltinMethod Integer class#opaqueIdentity>
        &foo_method_59, // #<BuiltinMethod Integer class#typecheck:>
    }
};

// #<Class Integer class>
struct FooClass foo_class_5 = {
    .name = &foo_string_55,
    .own_class = &foo_class_3,
    .methods = &foo_methods_61
};

// "Main"
struct FooBytes foo_string_62 = {
    .size = 4,
    .data = { 'M','a','i','n', 0 }
};

// #<BuiltinMethod Main#opaqueIdentity>
struct FooMethod foo_method_63 = {
    .home = &foo_class_6,
    .selector = &foo_selector_22,
    .method_function = foo_builtin_24
};

// "run:in:"
struct FooBytes foo_string_66 = {
    .size = 7,
    .data = { 'r','u','n',':','i','n',':', 0 }
};

// #run:in:
struct FooSelector foo_selector_65 = {
    .name = &foo_string_66
};

// #<CpsGraph Main#run:in:>
char* foo_method_function_67(char* sp, struct Actor* actor) {
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
struct FooMethod foo_method_64 = {
    .home = &foo_class_6,
    .selector = &foo_selector_65,
    .method_function = foo_method_function_67
};

// #<MethodDictionary Main>
struct FooMethodDictionary foo_methods_68 = {
    .size = 2,
    .data = {
        &foo_method_63, // #<BuiltinMethod Main#opaqueIdentity>
        &foo_method_64, // #<UserMethod Main#run:in:>
    }
};

// #<Class Main>
struct FooClass foo_class_6 = {
    .name = &foo_string_62,
    .own_class = &foo_class_7,
    .methods = &foo_methods_68
};

// "Main class"
struct FooBytes foo_string_69 = {
    .size = 10,
    .data = { 'M','a','i','n',' ','c','l','a','s','s', 0 }
};

// #<BuiltinMethodImpl new>
char* foo_builtin_71(char* sp, struct Actor* actor) {
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
struct FooMethod foo_method_70 = {
    .home = &foo_class_7,
    .selector = &foo_selector_28,
    .method_function = foo_builtin_71
};

// #<BuiltinMethod Main class#opaqueIdentity>
struct FooMethod foo_method_72 = {
    .home = &foo_class_7,
    .selector = &foo_selector_22,
    .method_function = foo_builtin_24
};

// #<BuiltinMethodImpl typecheck:>
char* foo_builtin_74(char* sp, struct Actor* actor) {
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
struct FooMethod foo_method_73 = {
    .home = &foo_class_7,
    .selector = &foo_selector_33,
    .method_function = foo_builtin_74
};

// #<MethodDictionary Main class>
struct FooMethodDictionary foo_methods_75 = {
    .size = 3,
    .data = {
        &foo_method_70, // #<BuiltinMethod Main class#new>
        &foo_method_72, // #<BuiltinMethod Main class#opaqueIdentity>
        &foo_method_73, // #<BuiltinMethod Main class#typecheck:>
    }
};

// #<Class Main class>
struct FooClass foo_class_7 = {
    .name = &foo_string_69,
    .own_class = &foo_class_3,
    .methods = &foo_methods_75
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
    PUSH_DATUM(sp, foo_method_function_67);
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
    printf(" INFO - exit\n");
    return 0;
}
