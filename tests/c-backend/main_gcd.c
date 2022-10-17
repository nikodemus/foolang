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

char* foo_continuation_80(char*, struct Actor*);
char* foo_continuation_83(char*, struct Actor*);
char* foo_continuation_85(char*, struct Actor*);
char* foo_continuation_86(char*, struct Actor*);
char* foo_continuation_90(char*, struct Actor*);

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

// "*"
struct FooBytes foo_string_41 = {
    .size = 1,
    .data = { '*', 0 }
};

// #*
struct FooSelector foo_selector_40 = {
    .name = &foo_string_41
};

// #<BuiltinMethodImpl *>
char* foo_builtin_42(char* sp, struct Actor* actor) {
    (void)sp;
    (void)actor;
    printf("TODO: #<CpsSelector *>");
    exit(1);
}

// #<BuiltinMethod Integer#*>
struct FooMethod foo_method_39 = {
    .home = &foo_class_4,
    .selector = &foo_selector_40,
    .method_function = foo_builtin_42
};

// "+"
struct FooBytes foo_string_45 = {
    .size = 1,
    .data = { '+', 0 }
};

// #+
struct FooSelector foo_selector_44 = {
    .name = &foo_string_45
};

// #<BuiltinMethodImpl +>
char* foo_builtin_46(char* sp, struct Actor* actor) {
    (void)sp;
    (void)actor;
    printf("TODO: #<CpsSelector +>");
    exit(1);
}

// #<BuiltinMethod Integer#+>
struct FooMethod foo_method_43 = {
    .home = &foo_class_4,
    .selector = &foo_selector_44,
    .method_function = foo_builtin_46
};

// "-"
struct FooBytes foo_string_49 = {
    .size = 1,
    .data = { '-', 0 }
};

// #-
struct FooSelector foo_selector_48 = {
    .name = &foo_string_49
};

// #<BuiltinMethodImpl ->
char* foo_builtin_50(char* sp, struct Actor* actor) {
    (void)sp;
    (void)actor;
    printf("TODO: #<CpsSelector ->");
    exit(1);
}

// #<BuiltinMethod Integer#->
struct FooMethod foo_method_47 = {
    .home = &foo_class_4,
    .selector = &foo_selector_48,
    .method_function = foo_builtin_50
};

// "//"
struct FooBytes foo_string_53 = {
    .size = 2,
    .data = { '/','/', 0 }
};

// #//
struct FooSelector foo_selector_52 = {
    .name = &foo_string_53
};

// #<BuiltinMethodImpl //>
char* foo_builtin_54(char* sp, struct Actor* actor) {
    (void)sp;
    (void)actor;
    printf("TODO: #<CpsSelector //>");
    exit(1);
}

// #<BuiltinMethod Integer#//>
struct FooMethod foo_method_51 = {
    .home = &foo_class_4,
    .selector = &foo_selector_52,
    .method_function = foo_builtin_54
};

// "addInteger:"
struct FooBytes foo_string_57 = {
    .size = 11,
    .data = { 'a','d','d','I','n','t','e','g','e','r',':', 0 }
};

// #addInteger:
struct FooSelector foo_selector_56 = {
    .name = &foo_string_57
};

// #<BuiltinMethodImpl addInteger:>
char* foo_builtin_58(char* sp, struct Actor* actor) {
    (void)sp;
    (void)actor;
    printf("TODO: #<CpsSelector addInteger:>");
    exit(1);
}

// #<BuiltinMethod Integer#addInteger:>
struct FooMethod foo_method_55 = {
    .home = &foo_class_4,
    .selector = &foo_selector_56,
    .method_function = foo_builtin_58
};

// "mulInteger:"
struct FooBytes foo_string_61 = {
    .size = 11,
    .data = { 'm','u','l','I','n','t','e','g','e','r',':', 0 }
};

// #mulInteger:
struct FooSelector foo_selector_60 = {
    .name = &foo_string_61
};

// #<BuiltinMethodImpl mulInteger:>
char* foo_builtin_62(char* sp, struct Actor* actor) {
    (void)sp;
    (void)actor;
    printf("TODO: #<CpsSelector mulInteger:>");
    exit(1);
}

// #<BuiltinMethod Integer#mulInteger:>
struct FooMethod foo_method_59 = {
    .home = &foo_class_4,
    .selector = &foo_selector_60,
    .method_function = foo_builtin_62
};

// #<BuiltinMethod Integer#opaqueIdentity>
struct FooMethod foo_method_63 = {
    .home = &foo_class_4,
    .selector = &foo_selector_22,
    .method_function = foo_builtin_24
};

// "subInteger:"
struct FooBytes foo_string_66 = {
    .size = 11,
    .data = { 's','u','b','I','n','t','e','g','e','r',':', 0 }
};

// #subInteger:
struct FooSelector foo_selector_65 = {
    .name = &foo_string_66
};

// #<BuiltinMethodImpl subInteger:>
char* foo_builtin_67(char* sp, struct Actor* actor) {
    (void)sp;
    (void)actor;
    printf("TODO: #<CpsSelector subInteger:>");
    exit(1);
}

// #<BuiltinMethod Integer#subInteger:>
struct FooMethod foo_method_64 = {
    .home = &foo_class_4,
    .selector = &foo_selector_65,
    .method_function = foo_builtin_67
};

// "truncateInteger:"
struct FooBytes foo_string_70 = {
    .size = 16,
    .data = { 't','r','u','n','c','a','t','e','I','n','t','e','g','e','r',':', 0 }
};

// #truncateInteger:
struct FooSelector foo_selector_69 = {
    .name = &foo_string_70
};

// #<BuiltinMethodImpl truncateInteger:>
char* foo_builtin_71(char* sp, struct Actor* actor) {
    (void)sp;
    (void)actor;
    printf("TODO: #<CpsSelector truncateInteger:>");
    exit(1);
}

// #<BuiltinMethod Integer#truncateInteger:>
struct FooMethod foo_method_68 = {
    .home = &foo_class_4,
    .selector = &foo_selector_69,
    .method_function = foo_builtin_71
};

// #<MethodDictionary Integer>
struct FooMethodDictionary foo_methods_72 = {
    .size = 9,
    .data = {
        &foo_method_39, // #<BuiltinMethod Integer#*>
        &foo_method_43, // #<BuiltinMethod Integer#+>
        &foo_method_47, // #<BuiltinMethod Integer#->
        &foo_method_51, // #<BuiltinMethod Integer#//>
        &foo_method_55, // #<BuiltinMethod Integer#addInteger:>
        &foo_method_59, // #<BuiltinMethod Integer#mulInteger:>
        &foo_method_63, // #<BuiltinMethod Integer#opaqueIdentity>
        &foo_method_64, // #<BuiltinMethod Integer#subInteger:>
        &foo_method_68, // #<BuiltinMethod Integer#truncateInteger:>
    }
};

// #<Class Integer>
struct FooClass foo_class_4 = {
    .name = &foo_string_38,
    .own_class = &foo_class_5,
    .methods = &foo_methods_72
};

// "Integer class"
struct FooBytes foo_string_73 = {
    .size = 13,
    .data = { 'I','n','t','e','g','e','r',' ','c','l','a','s','s', 0 }
};

// #<BuiltinMethod Integer class#opaqueIdentity>
struct FooMethod foo_method_74 = {
    .home = &foo_class_5,
    .selector = &foo_selector_22,
    .method_function = foo_builtin_24
};

// #<BuiltinMethodImpl typecheck:>
char* foo_builtin_76(char* sp, struct Actor* actor) {
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
struct FooMethod foo_method_75 = {
    .home = &foo_class_5,
    .selector = &foo_selector_29,
    .method_function = foo_builtin_76
};

// #<MethodDictionary Integer class>
struct FooMethodDictionary foo_methods_77 = {
    .size = 2,
    .data = {
        &foo_method_74, // #<BuiltinMethod Integer class#opaqueIdentity>
        &foo_method_75, // #<BuiltinMethod Integer class#typecheck:>
    }
};

// #<Class Integer class>
struct FooClass foo_class_5 = {
    .name = &foo_string_73,
    .own_class = &foo_class_3,
    .methods = &foo_methods_77
};

// "Main"
struct FooBytes foo_string_78 = {
    .size = 4,
    .data = { 'M','a','i','n', 0 }
};

// "gcd:of:"
struct FooBytes foo_string_81 = {
    .size = 7,
    .data = { 'g','c','d',':','o','f',':', 0 }
};

// #gcd:of:
struct FooSelector foo_selector_80 = {
    .name = &foo_string_81
};

// #<CpsGraph Main#gcd:of:>
char* foo_method_function_82(char* sp, struct Actor* actor) {
    // 0 - $return
    // 1 - $receiver class
    // 2 - $receiver datum
    // 3 - a class
    // 4 - a datum
    // 5 - b class
    // 6 - b datum
    // 7 - layout word
    // -
    // 8 - $ignore class
    // 9 - $ignore datum
    // 10 - $ret class
    // 11 - $ret datum
    // 12 - $ignore class
    // 13 - $ignore datum
    // 14 - $ret class
    // 15 - $ret datum
    // 16 - $ret class
    // 17 - $ret datum
    // 18 - $ret class
    // 19 - $ret datum
    // 20 - layout word
    char* bp = actor->bp = sp - 8 * sizeof(datum_t);
    if (READ_DATUM(bp, 3) != (datum_t)&foo_class_4) // a :: Integer
        return runtime_type_error(sp, actor);
    if (READ_DATUM(bp, 5) != (datum_t)&foo_class_4) // b :: Integer
        return runtime_type_error(sp, actor);
    PUSH_DATUM(sp, 0);
    PUSH_DATUM(sp, 0);
    PUSH_DATUM(sp, 0);
    PUSH_DATUM(sp, 0);
    PUSH_DATUM(sp, 0);
    PUSH_DATUM(sp, 0);
    PUSH_DATUM(sp, 0);
    PUSH_DATUM(sp, 0);
    PUSH_DATUM(sp, 0);
    PUSH_DATUM(sp, 0);
    PUSH_DATUM(sp, 0);
    PUSH_DATUM(sp, 0);
    PUSH_DATUM(sp, OBJS(6));
    PUSH_DATUM(sp, foo_continuation_83);
    PUSH_DATUM(sp, OBJS(1));
    if (((int64_t)READ_DATUM(bp, 4) == (int64_t)0)) {
        PUSH_DATUM(sp, foo_continuation_85);
    } else {
        PUSH_DATUM(sp, foo_continuation_86);
    }
    return sp;
}

// #<Continuation $seq:53>
char* foo_continuation_87(char* sp, struct Actor* actor) {
    char* bp = actor->bp;
    WRITE_DATUM(bp, 8, READ_DATUM(sp, -3));
    WRITE_DATUM(bp, 9, READ_DATUM(sp, -2));
    datum_t d0 = READ_DATUM(bp, 0);
    datum_t c1 = READ_DATUM(bp, 1);
    datum_t d2 = READ_DATUM(bp, 2);
    datum_t d4 = READ_DATUM(bp, 4);
    datum_t c5 = READ_DATUM(bp, 5);
    datum_t d6 = READ_DATUM(bp, 6);
    sp = bp;
    PUSH_DATUM(sp, d0);
    PUSH_DATUM(sp, c1);
    PUSH_DATUM(sp, d2);
    PUSH_DATUM(sp, c5);
    PUSH_DATUM(sp, d6);
    PUSH_DATUM(sp, &foo_class_4);
    PUSH_DATUM(sp, ((int64_t)d4 - (int64_t)((int64_t)d6 * (int64_t)((int64_t)d4 / (int64_t)d6))));
    PUSH_DATUM(sp, IMMS(1) | OBJS(3));
    PUSH_DATUM(sp, foo_method_function_82); // Main#gcd:of:
    return sp;
}

// #<Continuation $block:63>
char* foo_continuation_88(char* sp, struct Actor* actor) {
    char* bp = actor->bp;
    WRITE_DATUM(bp, 10, READ_DATUM(sp, -3));
    WRITE_DATUM(bp, 11, READ_DATUM(sp, -2));
    datum_t d0 = READ_DATUM(bp, 0);
    datum_t c3 = READ_DATUM(bp, 3);
    datum_t d4 = READ_DATUM(bp, 4);
    sp = bp;
    PUSH_DATUM(sp, c3);
    PUSH_DATUM(sp, d4);
    PUSH_DATUM(sp, OBJS(1));
    PUSH_DATUM(sp, d0);
    return sp;
}

// #<Continuation $seq:75>
char* foo_continuation_83(char* sp, struct Actor* actor) {
    char* bp = actor->bp;
    WRITE_DATUM(bp, 12, READ_DATUM(sp, -3));
    WRITE_DATUM(bp, 13, READ_DATUM(sp, -2));
    PUSH_DATUM(sp, foo_continuation_87);
    PUSH_DATUM(sp, OBJS(1));
    if (((int64_t)READ_DATUM(bp, 6) == (int64_t)0)) {
        PUSH_DATUM(sp, foo_continuation_88);
    } else {
        PUSH_DATUM(sp, foo_continuation_90);
    }
    return sp;
}

// #<Continuation $block:84>
char* foo_continuation_85(char* sp, struct Actor* actor) {
    char* bp = actor->bp;
    WRITE_DATUM(bp, 14, READ_DATUM(sp, -3));
    WRITE_DATUM(bp, 15, READ_DATUM(sp, -2));
    datum_t d0 = READ_DATUM(bp, 0);
    datum_t c5 = READ_DATUM(bp, 5);
    datum_t d6 = READ_DATUM(bp, 6);
    sp = bp;
    PUSH_DATUM(sp, c5);
    PUSH_DATUM(sp, d6);
    PUSH_DATUM(sp, OBJS(1));
    PUSH_DATUM(sp, d0);
    return sp;
}

// #<Continuation $ifFalse:115>
char* foo_continuation_86(char* sp, struct Actor* actor) {
    char* bp = actor->bp;
    WRITE_DATUM(bp, 16, READ_DATUM(sp, -3));
    WRITE_DATUM(bp, 17, READ_DATUM(sp, -2));
    datum_t c16 = READ_DATUM(bp, 16);
    datum_t d17 = READ_DATUM(bp, 17);
    PUSH_DATUM(sp, &foo_class_1);
    PUSH_DATUM(sp, 0); // FIXME: False?
    PUSH_DATUM(sp, OBJS(1));
    PUSH_DATUM(sp, c16); // FIXME: this doesn't look right
    PUSH_DATUM(sp, d17);
    return sp;
}

// #<Continuation $ifFalse:118>
char* foo_continuation_90(char* sp, struct Actor* actor) {
    char* bp = actor->bp;
    WRITE_DATUM(bp, 18, READ_DATUM(sp, -3));
    WRITE_DATUM(bp, 19, READ_DATUM(sp, -2));
    datum_t c18 = READ_DATUM(bp, 18);
    datum_t d19 = READ_DATUM(bp, 19);
    PUSH_DATUM(sp, &foo_class_1);
    PUSH_DATUM(sp, 0); // FIXME: False?
    PUSH_DATUM(sp, OBJS(1));
    PUSH_DATUM(sp, c18); // FIXME: this doesn't look right
    PUSH_DATUM(sp, d19);
    return sp;
}

// #<UserMethod Main#gcd:of:>
struct FooMethod foo_method_79 = {
    .home = &foo_class_6,
    .selector = &foo_selector_80,
    .method_function = foo_method_function_82
};

// #<BuiltinMethod Main#opaqueIdentity>
struct FooMethod foo_method_91 = {
    .home = &foo_class_6,
    .selector = &foo_selector_22,
    .method_function = foo_builtin_24
};

// "run:in:"
struct FooBytes foo_string_94 = {
    .size = 7,
    .data = { 'r','u','n',':','i','n',':', 0 }
};

// #run:in:
struct FooSelector foo_selector_93 = {
    .name = &foo_string_94
};

// #<CpsGraph Main#run:in:>
char* foo_method_function_95(char* sp, struct Actor* actor) {
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
struct FooMethod foo_method_92 = {
    .home = &foo_class_6,
    .selector = &foo_selector_93,
    .method_function = foo_method_function_95
};

// #<MethodDictionary Main>
struct FooMethodDictionary foo_methods_96 = {
    .size = 3,
    .data = {
        &foo_method_79, // #<UserMethod Main#gcd:of:>
        &foo_method_91, // #<BuiltinMethod Main#opaqueIdentity>
        &foo_method_92, // #<UserMethod Main#run:in:>
    }
};

// #<Class Main>
struct FooClass foo_class_6 = {
    .name = &foo_string_78,
    .own_class = &foo_class_7,
    .methods = &foo_methods_96
};

// "Main class"
struct FooBytes foo_string_97 = {
    .size = 10,
    .data = { 'M','a','i','n',' ','c','l','a','s','s', 0 }
};

// "new"
struct FooBytes foo_string_100 = {
    .size = 3,
    .data = { 'n','e','w', 0 }
};

// #new
struct FooSelector foo_selector_99 = {
    .name = &foo_string_100
};

// #<BuiltinMethodImpl new>
char* foo_builtin_101(char* sp, struct Actor* actor) {
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
struct FooMethod foo_method_98 = {
    .home = &foo_class_7,
    .selector = &foo_selector_99,
    .method_function = foo_builtin_101
};

// #<BuiltinMethod Main class#opaqueIdentity>
struct FooMethod foo_method_102 = {
    .home = &foo_class_7,
    .selector = &foo_selector_22,
    .method_function = foo_builtin_24
};

// #<BuiltinMethodImpl typecheck:>
char* foo_builtin_104(char* sp, struct Actor* actor) {
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
struct FooMethod foo_method_103 = {
    .home = &foo_class_7,
    .selector = &foo_selector_29,
    .method_function = foo_builtin_104
};

// #<MethodDictionary Main class>
struct FooMethodDictionary foo_methods_105 = {
    .size = 3,
    .data = {
        &foo_method_98, // #<BuiltinMethod Main class#new>
        &foo_method_102, // #<BuiltinMethod Main class#opaqueIdentity>
        &foo_method_103, // #<BuiltinMethod Main class#typecheck:>
    }
};

// #<Class Main class>
struct FooClass foo_class_7 = {
    .name = &foo_string_97,
    .own_class = &foo_class_3,
    .methods = &foo_methods_105
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
    PUSH_DATUM(sp, foo_method_function_95);
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
