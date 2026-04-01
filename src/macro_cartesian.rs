#[macro_export]
macro_rules! cartesian {
    // ==========================================
    // 1. 拦截 `<` 并开启图层解析 (Token Muncher)
    // ==========================================
    (@parse_gen
        input: [ < $($tail:tt)* ]
        generics: [ $($g_acc:tt)* ]
    ) => {
        cartesian!(@parse_layer
            acc: { decl: [], decl_comma: [], use_comma: [] }
            tail: [ $($tail)* ]
            generics: [ $($g_acc)* ]
        );
    };

    (@parse_layer
        acc: { decl: [$($d:tt)*], decl_comma: [$($dc:tt)*], use_comma: [$($uc:tt)*] }
        tail: [ _ , $($tail:tt)* ]
        generics: [ $($g_acc:tt)* ]
    ) => {
        cartesian!(@parse_layer
            acc: { decl: [$($d)*], decl_comma: [$($dc)*], use_comma: [$($uc)*] }
            tail: [ $($tail)* ]
            generics: [ $($g_acc)* ]
        );
    };

    (@parse_layer
        acc: { decl: [$($d:tt)*], decl_comma: [$($dc:tt)*], use_comma: [$($uc:tt)*] }
        tail: [ _ > $($tail:tt)* ]
        generics: [ $($g_acc:tt)* ]
    ) => {
        cartesian!(@parse_gen
            input: [ $($tail)* ]
            generics: [ $($g_acc)* { decl: [$($d)*], decl_comma: [$($dc)*], use_comma: [$($uc)*] } ]
        );
    };

    (@parse_layer
        acc: { decl: [$($d:tt)*], decl_comma: [$($dc:tt)*], use_comma: [$($uc:tt)*] }
        tail: [ const $N:ident : $t:ty , $($tail:tt)* ]
        generics: [ $($g_acc:tt)* ]
    ) => {
        cartesian!(@parse_layer
            acc: {
                decl: [$($d)* const $N: $t ,],
                decl_comma: [$($dc)* const $N: $t ,],
                use_comma: [$($uc)* $N ,]
            }
            tail: [ $($tail)* ]
            generics: [ $($g_acc)* ]
        );
    };

    (@parse_layer
        acc: { decl: [$($d:tt)*], decl_comma: [$($dc:tt)*], use_comma: [$($uc:tt)*] }
        tail: [ const $N:ident : $t:ty > $($tail:tt)* ]
        generics: [ $($g_acc:tt)* ]
    ) => {
        cartesian!(@parse_gen
            input: [ $($tail)* ]
            generics: [ $($g_acc)* {
                decl: [$($d)* const $N: $t],
                decl_comma: [$($dc)* const $N: $t ,],
                use_comma: [$($uc)* $N ,]
            } ]
        );
    };

    (@parse_layer
        acc: { decl: [$($d:tt)*], decl_comma: [$($dc:tt)*], use_comma: [$($uc:tt)*] }
        tail: [ $T:ident : $bound:path , $($tail:tt)* ]
        generics: [ $($g_acc:tt)* ]
    ) => {
        cartesian!(@parse_layer
            acc: {
                decl: [$($d)* $T: $bound ,],
                decl_comma: [$($dc)* $T: $bound ,],
                use_comma: [$($uc)* $T ,]
            }
            tail: [ $($tail)* ]
            generics: [ $($g_acc)* ]
        );
    };

    (@parse_layer
        acc: { decl: [$($d:tt)*], decl_comma: [$($dc:tt)*], use_comma: [$($uc:tt)*] }
        tail: [ $T:ident : $bound:path > $($tail:tt)* ]
        generics: [ $($g_acc:tt)* ]
    ) => {
        cartesian!(@parse_gen
            input: [ $($tail)* ]
            generics: [ $($g_acc)* {
                decl: [$($d)* $T: $bound],
                decl_comma: [$($dc)* $T: $bound ,],
                use_comma: [$($uc)* $T ,]
            } ]
        );
    };

    (@parse_layer
        acc: { decl: [$($d:tt)*], decl_comma: [$($dc:tt)*], use_comma: [$($uc:tt)*] }
        tail: [ $T:ident , $($tail:tt)* ]
        generics: [ $($g_acc:tt)* ]
    ) => {
        cartesian!(@parse_layer
            acc: {
                decl: [$($d)* $T ,],
                decl_comma: [$($dc)* $T ,],
                use_comma: [$($uc)* $T ,]
            }
            tail: [ $($tail)* ]
            generics: [ $($g_acc)* ]
        );
    };

    (@parse_layer
        acc: { decl: [$($d:tt)*], decl_comma: [$($dc:tt)*], use_comma: [$($uc:tt)*] }
        tail: [ $T:ident > $($tail:tt)* ]
        generics: [ $($g_acc:tt)* ]
    ) => {
        cartesian!(@parse_gen
            input: [ $($tail)* ]
            generics: [ $($g_acc)* {
                decl: [$($d)* $T],
                decl_comma: [$($dc)* $T ,],
                use_comma: [$($uc)* $T ,]
            } ]
        );
    };

    (@parse_layer
        acc: { decl: [$($d:tt)*], decl_comma: [$($dc:tt)*], use_comma: [$($uc:tt)*] }
        tail: [ > $($tail:tt)* ]
        generics: [ $($g_acc:tt)* ]
    ) => {
        cartesian!(@parse_gen
            input: [ $($tail)* ]
            generics: [ $($g_acc)* { decl: [$($d)*], decl_comma: [$($dc)*], use_comma: [$($uc)*] } ]
        );
    };

    // ==========================================
    // 3. Zip 阶段：将 泛型、变量、函数、对应的 Handler 对齐
    // ==========================================
    (@parse_gen
        // 修复：$f 改为 ident 避免 `:` 的跟随集冲突
        input: [ ( $($var:ident : $ty:ty),+ ) $body:block ; $($f:ident : $h:path),+ ]
        generics: [ $($g_acc:tt)* ]
    ) => {
        cartesian!(@zip
            generics: [ $($g_acc)* ]
            vars: [ $( ($var, $ty) )+ ]
            funcs_and_handlers: [ $( { f: $f, h: $h } )+ ]
            acc: []
            body: $body
        );
    };

    (@zip
        generics: [ { decl: [$($decl:tt)*], decl_comma: [$($dc:tt)*], use_comma: [$($uc:tt)*] } $($g_rest:tt)* ]
        vars: [ ($v:ident, $ty:ty) $($v_rest:tt)* ]
        // 修复：这里也对应改成 $f:ident
        funcs_and_handlers: [ { f: $f:ident, h: $h:path } $($fh_rest:tt)* ]
        acc: [ $($acc:tt)* ]
        body: $body:block
    ) => {
        cartesian!(@zip
            generics: [ $($g_rest)* ]
            vars: [ $($v_rest)* ]
            funcs_and_handlers: [ $($fh_rest)* ]
            acc: [ $($acc)* {
                decl: [$($decl)*], decl_comma: [$($dc)*], use_comma: [$($uc)*],
                var: $v, ty: $ty, f: $f, h: $h
            } ]
            body: $body
        );
    };

    (@zip
        generics: []
        vars: []
        funcs_and_handlers: []
        acc: [ $($parsed:tt)+ ]
        body: $body:block
    ) => {
        cartesian!(@start
            remaining: [ $($parsed)+ ]
            body: $body
        );
    };

    // ==========================================
    // 4. 嵌套结构体生成
    // ==========================================
    (@start
        remaining: [
            // 修复：这里也对应改成 $f:ident
            { decl: [$($decl:tt)*], decl_comma: [$($dc:tt)*], use_comma: [$($uc:tt)*], var: $var:ident, ty: $ty:ty, f: $f:ident, h: $h:path }
        ]
        body: $body:block
    ) => {{
        struct L;
        cartesian!(@impl_trait
            handler: $h, // <--- 修复：加上逗号
            struct_name: L generics_decl: [] generics_use: []
            method_decl: [$($decl)*] method_var: $var method_ty: $ty,
            extract: [],
            method_body: { $body }
        );
        $f(&mut L)
    }};

    (@start
        remaining: [
            { decl: [$($decl:tt)*], decl_comma: [$($dc:tt)*], use_comma: [$($uc:tt)*], var: $var:ident, ty: $ty:ty, f: $f:ident, h: $h:path }
            $( $rest:tt )+
        ]
        body: $body:block
    ) => {{
        struct L;
        cartesian!(@impl_trait
            handler: $h, // <--- 修复：加上逗号
            struct_name: L generics_decl: [] generics_use: []
            method_decl: [$($decl)*] method_var: $var method_ty: $ty,
            extract: [],
            method_body: {
                cartesian!(@nest
                    acc_decl: [ $($dc)* ]
                    acc_use: [ $($uc)* ]
                    self_fields: []
                    param: { var: $var, ty: $ty }
                    remaining: [ $($rest)+ ]
                    body: $body
                )
            }
        );
        $f(&mut L)
    }};

    (@nest
        acc_decl: [ $($acc_d:tt)* ]
        acc_use: [ $($acc_u:tt)* ]
        self_fields: [ $( { var: $s_var:ident, ty: $s_ty:ty } )* ]
        param: { var: $p_var:ident, ty: $p_ty:ty }
        remaining: [
            { decl: [$($decl:tt)*], decl_comma: [$($dc:tt)*], use_comma: [$($uc:tt)*], var: $var:ident, ty: $ty:ty, f: $f:ident, h: $h:path }
        ]
        body: $body:block
    ) => {{
        cartesian!(@def_struct
            struct_name: L generics_decl: [ $($acc_d)* ]
            fields: {
                $( $s_var : $s_ty , )*
                $p_var : $p_ty,
            }
        );
        cartesian!(@impl_trait
            handler: $h, // <--- 修复：加上逗号
            struct_name: L generics_decl: [ $($acc_d)* ] generics_use: [ $($acc_u)* ]
            method_decl: [ $($decl)* ] method_var: $var method_ty: $ty,
            extract: [ $($s_var)* $p_var ],
            method_body: {
                $body
            }
        );
        $f(&mut L {
            $( $s_var, )*
            $p_var,
        })
    }};

    (@nest
        acc_decl: [ $($acc_d:tt)* ]
        acc_use: [ $($acc_u:tt)* ]
        self_fields: [ $( { var: $s_var:ident, ty: $s_ty:ty } )* ]
        param: { var: $p_var:ident, ty: $p_ty:ty }
        remaining: [
            { decl: [$($decl:tt)*], decl_comma: [$($dc:tt)*], use_comma: [$($uc:tt)*], var: $var:ident, ty: $ty:ty, f: $f:ident, h: $h:path }
            $( $rest:tt )+
        ]
        body: $body:block
    ) => {{
        cartesian!(@def_struct
            struct_name: L generics_decl: [ $($acc_d)* ]
            fields: {
                $( $s_var : $s_ty , )*
                $p_var : $p_ty,
            }
        );
        cartesian!(@impl_trait
            handler: $h, // <--- 修复：加上逗号
            struct_name: L generics_decl: [ $($acc_d)* ] generics_use: [ $($acc_u)* ]
            method_decl: [ $($decl)* ] method_var: $var method_ty: $ty,
            extract: [ $($s_var)* $p_var ],
            method_body: {
                cartesian!(@nest
                    acc_decl: [ $($acc_d)* $($dc)* ]
                    acc_use: [ $($acc_u)* $($uc)* ]
                    self_fields: [
                        $( { var: $s_var, ty: $s_ty } )*
                        { var: $p_var, ty: $p_ty }
                    ]
                    param: { var: $var, ty: $ty }
                    remaining: [ $($rest)+ ]
                    body: $body
                )
            }
        );
        $f(&mut L {
            $( $s_var, )*
            $p_var,
        })
    }};

    // ==========================================
    // 助手宏
    // ==========================================
    (@def_struct struct_name: $name:ident generics_decl: [] fields: { $($fields:tt)* }) => {
        struct $name { $($fields)* }
    };
    (@def_struct struct_name: $name:ident generics_decl: [ $($g:tt)+ ] fields: { $($fields:tt)* }) => {
        struct $name < $($g)* > { $($fields)* }
    };

    (@impl_trait
        handler: $h:path, // <--- 修复：加上逗号
        struct_name: $name:ident generics_decl: [] generics_use: []
        method_decl: [$($m_decl:tt)*] method_var: $var:ident method_ty: $ty:ty,
        extract: [ $($ext:ident)* ],
        method_body: $body:block
    ) => {
        impl $h for $name {
            cartesian!(@def_method decl: [$($m_decl)*] var: $var ty: $ty, extract: [$($ext)*], body: $body);
        }
    };
    (@impl_trait
        handler: $h:path, // <--- 修复：加上逗号
        struct_name: $name:ident generics_decl: [ $($g_decl:tt)+ ] generics_use: [ $($g_use:tt)+ ]
        method_decl: [$($m_decl:tt)*] method_var: $var:ident method_ty: $ty:ty,
        extract: [ $($ext:ident)* ],
        method_body: $body:block
    ) => {
        impl < $($g_decl)* > $h for $name < $($g_use)* > {
            cartesian!(@def_method decl: [$($m_decl)*] var: $var ty: $ty, extract: [$($ext)*], body: $body);
        }
    };

    (@def_method decl: [] var: $var:ident ty: $ty:ty, extract: [ $($ext:ident)* ], body: $body:block) => {
        fn call(&mut self, $var: $ty) {
            $( let $ext = self.$ext.clone(); )*
            $body
        }
    };
    (@def_method decl: [ $($m_decl:tt)+ ] var: $var:ident ty: $ty:ty, extract: [ $($ext:ident)* ], body: $body:block) => {
        fn call< $($m_decl)+ >(&mut self, $var: $ty) {
            $( let $ext = self.$ext.clone(); )*
            $body
        }
    };

    // ==========================================
    // 入口点
    // ==========================================
    ( $($input:tt)* ) => {
        cartesian!(@parse_gen
            input: [ $($input)* ]
            generics: []
        );
    };
}

#[cfg(test)]
mod test {
    trait OuterHandler {
        fn call<const N: usize>(&mut self, x: [usize; N]);
    }

    trait InnerHandler {
        fn call(&mut self, x: &'static str);
    }

    fn f1<H: OuterHandler>(h: &mut H) {
        h.call([1, 2, 3]);
        h.call([4, 5]);
    }

    fn f2<H: InnerHandler>(h: &mut H) {
        h.call("hello");
        h.call("macro");
    }

    #[test]
    fn main() {
        cartesian!(
            <const N1: usize>
            <_>
            (a: [usize; N1], b: &'static str) {
                println!("a = {:?}, b = {:?}", a, b);
            };
            f1: OuterHandler, f2: InnerHandler
        );
    }
}
