mod logger_db;
use logger_db::LoggerDb;

#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct InternedStringId<'db>(
    salsa::Id,
    std::marker::PhantomData<&'db salsa::plumbing::interned::Value<InternedStringId<'static>>>,
);

const _: () = {
    use salsa::plumbing as zalsa_;
    use zalsa_::interned as zalsa_struct_;
    type Configuration_ = InternedStringId<'static>;
    type StructData<'db> = (InternedString,);
    /// Key to use during hash lookups. Each field is some type that implements `Lookup<T>`
    /// for the owned type. This permits interning with an `&str` when a `String` is required and so forth.
    struct StructKey<'db, T0: zalsa_::interned::Lookup<InternedString>>(
        T0,
        std::marker::PhantomData<&'db ()>,
    );
    impl<'db, T0: zalsa_::interned::Lookup<InternedString>>
        zalsa_::interned::Lookup<StructData<'db>> for StructKey<'db, T0>
    {
        fn hash<H: std::hash::Hasher>(&self, h: &mut H) {
            zalsa_::interned::Lookup::hash(&self.0, &mut *h);
        }
        fn eq(&self, data: &StructData<'db>) -> bool {
            (zalsa_::interned::Lookup::eq(&self.0, &data.0) && true)
        }
        #[allow(unused_unit)]
        fn into_owned(self) -> StructData<'db> {
            (zalsa_::interned::Lookup::into_owned(self.0),)
        }
    }
    impl zalsa_struct_::Configuration for Configuration_ {
        const DEBUG_NAME: &'static str = "InternedStringId";
        type Data<'a> = StructData<'a>;
        type Struct<'a> = InternedStringId<'a>;
        fn struct_from_id<'db>(id: salsa::Id) -> Self::Struct<'db> {
            InternedStringId(id, std::marker::PhantomData)
        }
        fn deref_struct(s: Self::Struct<'_>) -> salsa::Id {
            s.0
        }
    }
    impl Configuration_ {
        pub fn ingredient<Db>(db: &Db) -> &zalsa_struct_::IngredientImpl<Self>
        where
            Db: ?Sized + zalsa_::Database,
        {
            static CACHE: zalsa_::IngredientCache<zalsa_struct_::IngredientImpl<Configuration_>> =
                zalsa_::IngredientCache::new();
            CACHE.get_or_create(db.as_dyn_database(), || {
                db.zalsa()
                    .add_or_lookup_jar_by_type(&<zalsa_struct_::JarImpl<Configuration_>>::default())
            })
        }
    }
    impl zalsa_::AsId for InternedStringId<'_> {
        fn as_id(&self) -> salsa::Id {
            self.0
        }
    }
    impl zalsa_::FromId for InternedStringId<'_> {
        fn from_id(id: salsa::Id) -> Self {
            Self(id, std::marker::PhantomData)
        }
    }
    unsafe impl Send for InternedStringId<'_> {}
    unsafe impl Sync for InternedStringId<'_> {}
    impl std::fmt::Debug for InternedStringId<'_> {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            Self::default_debug_fmt(*self, f)
        }
    }
    impl zalsa_::SalsaStructInDb for InternedStringId<'_> {}
    unsafe impl zalsa_::Update for InternedStringId<'_> {
        unsafe fn maybe_update(old_pointer: *mut Self, new_value: Self) -> bool {
            if unsafe { *old_pointer } != new_value {
                unsafe { *old_pointer = new_value };
                true
            } else {
                false
            }
        }
    }
    impl<'db> InternedStringId<'db> {
        pub fn new<Db_>(db: &'db Db_, data: impl zalsa_::interned::Lookup<InternedString>) -> Self
        where
            Db_: ?Sized + salsa::Database,
        {
            let current_revision = zalsa_::current_revision(db);
            Configuration_::ingredient(db).intern(
                db.as_dyn_database(),
                StructKey::<'db>(data, std::marker::PhantomData::default()),
            )
        }
        fn data<Db_>(self, db: &'db Db_) -> InternedString
        where
            Db_: ?Sized + zalsa_::Database,
        {
            let fields = Configuration_::ingredient(db).fields(db.as_dyn_database(), self);
            std::clone::Clone::clone(&fields.0)
        }
        /// Default debug formatting for this struct (may be useful if you define your own `Debug` impl)
        pub fn default_debug_fmt(this: Self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            zalsa_::with_attached_database(|db| {
                let fields = Configuration_::ingredient(db).fields(db.as_dyn_database(), this);
                let mut f = f.debug_struct("InternedStringId");
                let f = f.field("data", &fields.0);
                f.finish()
            })
            .unwrap_or_else(|| {
                f.debug_tuple("InternedStringId")
                    .field(&zalsa_::AsId::as_id(&this))
                    .finish()
            })
        }
    }
};

#[derive(Clone, PartialEq, Eq, Debug, Hash)]
pub struct InternedString {
    data: String,
}

pub trait InternedDB: salsa::Database {
    fn string(&self) -> String;
    fn intern_string(&self, data: InternedString) -> InternedStringId<'_>;
    fn interned_len<'db>(&self, id: InternedStringId<'db>) -> usize;
    #[doc(hidden)]
    fn zalsa_db(&self);
}
impl<DB> InternedDB for DB
where
    DB: salsa::Database,
{
    fn string(&self) -> String {
        let data = create_data_InternedDB(self);
        data.string(self).unwrap()
    }
    fn intern_string(&self, data: InternedString) -> InternedStringId<'_> {
        InternedStringId::new(self, data)
    }
    fn interned_len<'db>(&self, id: InternedStringId<'db>) -> usize {
        #[allow(clippy::needless_lifetimes)]
        fn interned_len_shim<'db>(
            db: &'db dyn InternedDB,
            _input: InternedDBData,
            id: InternedStringId<'db>,
        ) -> usize {
            use salsa::plumbing as zalsa_;
            struct Configuration_;
            static FN_CACHE_: zalsa_::IngredientCache<
                zalsa_::function::IngredientImpl<Configuration_>,
            > = zalsa_::IngredientCache::new();

            #[derive(Clone, Copy)]
            struct InternedData_<'db>(
                salsa::Id,
                std::marker::PhantomData<&'db zalsa_::interned::Value<Configuration_>>,
            );

            static INTERN_CACHE_: zalsa_::IngredientCache<
                zalsa_::interned::IngredientImpl<Configuration_>,
            > = zalsa_::IngredientCache::new();
            impl zalsa_::SalsaStructInDb for InternedData_<'_> {}
            impl zalsa_::interned::Configuration for Configuration_ {
                const DEBUG_NAME: &'static str = "Configuration";
                type Data<'db> = (InternedDBData, InternedStringId<'db>);
                type Struct<'db> = InternedData_<'db>;
                fn struct_from_id<'db>(id: salsa::Id) -> Self::Struct<'db> {
                    InternedData_(id, std::marker::PhantomData)
                }
                fn deref_struct(s: Self::Struct<'_>) -> salsa::Id {
                    s.0
                }
            }
            impl Configuration_ {
                fn fn_ingredient(
                    db: &dyn InternedDB,
                ) -> &zalsa_::function::IngredientImpl<Configuration_> {
                    FN_CACHE_.get_or_create(db.as_dyn_database(), || {
                        <dyn InternedDB as InternedDB>::zalsa_db(db);
                        db.zalsa().add_or_lookup_jar_by_type(&Configuration_)
                    })
                }
                fn intern_ingredient(
                    db: &dyn InternedDB,
                ) -> &zalsa_::interned::IngredientImpl<Configuration_> {
                    INTERN_CACHE_.get_or_create(db.as_dyn_database(), || {
                        db.zalsa()
                            .add_or_lookup_jar_by_type(&Configuration_)
                            .successor(0)
                    })
                }
            }
            impl zalsa_::function::Configuration for Configuration_ {
                const DEBUG_NAME: &'static str = "interned_len_shim";
                type DbView = dyn InternedDB;
                type SalsaStruct<'db> = InternedData_<'db>;
                type Input<'db> = (InternedDBData, InternedStringId<'db>);
                type Output<'db> = usize;
                const CYCLE_STRATEGY: zalsa_::CycleRecoveryStrategy =
                    zalsa_::CycleRecoveryStrategy::Panic;
                fn should_backdate_value(
                    old_value: &Self::Output<'_>,
                    new_value: &Self::Output<'_>,
                ) -> bool {
                    zalsa_::should_backdate_value(old_value, new_value)
                }
                fn execute<'db>(
                    db: &'db Self::DbView,
                    (_input, id): (InternedDBData, InternedStringId<'db>),
                ) -> Self::Output<'db> {
                    fn inner_<'db>(
                        db: &dyn InternedDB,
                        _input: InternedDBData,
                        id: InternedStringId<'db>,
                    ) -> usize {
                        interned_len(db, id)
                    }
                    inner_(db, _input, id)
                }
                fn recover_from_cycle<'db>(
                    db: &'db dyn InternedDB,
                    cycle: &zalsa_::Cycle,
                    (_input, id): (InternedDBData, InternedStringId<'db>),
                ) -> Self::Output<'db> {
                    {
                        std::mem::drop(db);
                        std::mem::drop((_input, id));
                        {
                            panic!("cannot recover from cycle `{0:?}`", cycle);
                        }
                    }
                }
                fn id_to_input<'db>(db: &'db Self::DbView, key: salsa::Id) -> Self::Input<'db> {
                    Configuration_::intern_ingredient(db)
                        .data(db.as_dyn_database(), key)
                        .clone()
                }
            }
            impl zalsa_::Jar for Configuration_ {
                fn create_ingredients(
                    &self,
                    aux: &dyn zalsa_::JarAux,
                    first_index: zalsa_::IngredientIndex,
                ) -> Vec<Box<dyn zalsa_::Ingredient>> {
                    let fn_ingredient =
                        <zalsa_::function::IngredientImpl<Configuration_>>::new(first_index, aux);
                    fn_ingredient.set_capacity(0);
                    <[_]>::into_vec(Box::new([
                        Box::new(fn_ingredient),
                        Box::new(<zalsa_::interned::IngredientImpl<Configuration_>>::new(
                            first_index.successor(0),
                        )),
                    ]))
                }
            }
            #[allow(non_local_definitions)]
            impl interned_len_shim {
                pub fn accumulated<'db, A: salsa::Accumulator>(
                    db: &'db dyn InternedDB,
                    _input: InternedDBData,
                    id: InternedStringId<'db>,
                ) -> Vec<A> {
                    use salsa::plumbing as zalsa_;
                    let key = Configuration_::intern_ingredient(db)
                        .intern_id(db.as_dyn_database(), (_input, id));
                    Configuration_::fn_ingredient(db).accumulated_by::<A>(db, key)
                }
            }
            zalsa_::attach(db, || {
                let result = {
                    let key = Configuration_::intern_ingredient(db)
                        .intern_id(db.as_dyn_database(), (_input, id));
                    Configuration_::fn_ingredient(db).fetch(db, key)
                };
                <usize as std::clone::Clone>::clone(result)
            })
        }
        #[allow(non_camel_case_types)]
        struct interned_len_shim {
            _priv: std::convert::Infallible,
        }
        interned_len_shim(self, create_data_InternedDB(self), id)
    }
    #[doc(hidden)]
    fn zalsa_db(&self) {
        use salsa::plumbing as zalsa_;
        zalsa_::views(self).add::<Self, dyn InternedDB>(|t| t);
    }
}

#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
struct InternedDBData(salsa::Id);

const _: () = {
    use salsa::plumbing as zalsa_;
    use zalsa_::input as zalsa_struct_;
    struct Configuration_;
    impl zalsa_struct_::Configuration for Configuration_ {
        const DEBUG_NAME: &'static str = "InternedDBData";
        const FIELD_DEBUG_NAMES: &'static [&'static str] = &["string"];
        const IS_SINGLETON: bool = false;
        /// The input struct (which wraps an `Id`)
        type Struct = InternedDBData;
        /// A (possibly empty) tuple of the fields for this struct.
        type Fields = (Option<String>,);
        /// A array of [`StampedValue<()>`](`StampedValue`) tuples, one per each of the value fields.
        type Stamps = zalsa_::Array<zalsa_::Stamp, 1>;
    }
    impl Configuration_ {
        pub fn ingredient(db: &dyn zalsa_::Database) -> &zalsa_struct_::IngredientImpl<Self> {
            static CACHE: zalsa_::IngredientCache<zalsa_struct_::IngredientImpl<Configuration_>> =
                zalsa_::IngredientCache::new();
            CACHE.get_or_create(db, || {
                db.zalsa()
                    .add_or_lookup_jar_by_type(&<zalsa_struct_::JarImpl<Configuration_>>::default())
            })
        }
        pub fn ingredient_mut(
            db: &mut dyn zalsa_::Database,
        ) -> (
            &mut zalsa_struct_::IngredientImpl<Self>,
            &mut zalsa_::Runtime,
        ) {
            let zalsa_mut = db.zalsa_mut();
            let index = zalsa_mut
                .add_or_lookup_jar_by_type(&<zalsa_struct_::JarImpl<Configuration_>>::default());
            let current_revision = zalsa_mut.current_revision();
            let (ingredient, runtime) = zalsa_mut.lookup_ingredient_mut(index);
            let ingredient = ingredient.assert_type_mut::<zalsa_struct_::IngredientImpl<Self>>();
            (ingredient, runtime)
        }
    }
    impl zalsa_::FromId for InternedDBData {
        fn from_id(id: salsa::Id) -> Self {
            Self(id)
        }
    }
    impl zalsa_::AsId for InternedDBData {
        fn as_id(&self) -> salsa::Id {
            self.0
        }
    }
    impl std::fmt::Debug for InternedDBData {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            Self::default_debug_fmt(*self, f)
        }
    }
    impl zalsa_::SalsaStructInDb for InternedDBData {}
    impl InternedDBData {
        #[inline]
        pub fn new<Db_>(db: &Db_, string: Option<String>) -> Self
        where
            Db_: ?Sized + salsa::Database,
        {
            Self::builder(string).new(db)
        }
        pub fn builder(string: Option<String>) -> <Self as zalsa_struct_::HasBuilder>::Builder {
            builder::new_builder(string)
        }
        fn string<'db, Db_>(self, db: &'db Db_) -> Option<String>
        where
            Db_: ?Sized + zalsa_::Database,
        {
            let fields = Configuration_::ingredient(db.as_dyn_database()).field(
                db.as_dyn_database(),
                self,
                0,
            );
            std::clone::Clone::clone(&fields.0)
        }
        #[must_use]
        fn set_string<'db, Db_>(
            self,
            db: &'db mut Db_,
        ) -> impl salsa::Setter<FieldTy = Option<String>> + 'db
        where
            Db_: ?Sized + zalsa_::Database,
        {
            let (ingredient, revision) = Configuration_::ingredient_mut(db.as_dyn_database_mut());
            zalsa_::input::SetterImpl::new(revision, self, 0, ingredient, |fields, f| {
                std::mem::replace(&mut fields.0, f)
            })
        }
        /// Default debug formatting for this struct (may be useful if you define your own `Debug` impl)
        pub fn default_debug_fmt(this: Self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            zalsa_::with_attached_database(|db| {
                let fields = Configuration_::ingredient(db).leak_fields(db, this);
                let mut f = f.debug_struct("InternedDBData");
                let f = f.field("[salsa id]", &zalsa_::AsId::as_id(&this));
                let f = f.field("string", &fields.0);
                f.finish()
            })
            .unwrap_or_else(|| {
                f.debug_struct("InternedDBData")
                    .field("[salsa id]", &this.0)
                    .finish()
            })
        }
    }
    impl zalsa_struct_::HasBuilder for InternedDBData {
        type Builder = builder::Builder_;
    }
    impl builder::Builder_ {
        /// Creates the new input with the set values.
        #[must_use]
        pub fn new<Db_>(self, db: &Db_) -> InternedDBData
        where
            Db_: ?Sized + salsa::Database,
        {
            let current_revision = zalsa_::current_revision(db);
            let ingredient = Configuration_::ingredient(db.as_dyn_database());
            let (fields, stamps) = builder::builder_into_inner(self, current_revision);
            ingredient.new_input(db.as_dyn_database(), fields, stamps)
        }
    }
    mod builder {
        use super::*;
        use salsa::plumbing as zalsa_;
        use zalsa_::input as zalsa_struct_;
        pub(super) fn new_builder(string: Option<String>) -> Builder_ {
            Builder_ {
                fields: (string,),
                durabilities: [salsa::Durability::default(); 1],
            }
        }
        pub(super) fn builder_into_inner(
            builder: Builder_,
            revision: zalsa_::Revision,
        ) -> ((Option<String>,), zalsa_::Array<zalsa_::Stamp, 1>) {
            let stamps = zalsa_::Array::new([zalsa_::stamp(revision, builder.durabilities[0])]);
            (builder.fields, stamps)
        }
        #[must_use]
        pub struct Builder_ {
            /// The field values.
            fields: (Option<String>,),
            /// The durabilities per field.
            durabilities: [salsa::Durability; 1],
        }
        impl Builder_ {
            /// Sets the durability of all fields.
            ///
            /// Overrides any previously set durabilities.
            pub fn durability(mut self, durability: salsa::Durability) -> Self {
                self.durabilities = [durability; 1];
                self
            }
            /// Sets the durability for the field `$field_id`.
            #[must_use]
            pub fn string_durability(mut self, durability: salsa::Durability) -> Self {
                self.durabilities[0] = durability;
                self
            }
        }
    }
};
#[allow(clippy::needless_lifetimes)]
#[allow(non_snake_case)]
fn create_data_InternedDB<'db>(db: &'db dyn InternedDB) -> InternedDBData {
    use salsa::plumbing as zalsa_;
    struct Configuration_;
    static FN_CACHE_: zalsa_::IngredientCache<zalsa_::function::IngredientImpl<Configuration_>> =
        zalsa_::IngredientCache::new();

    #[derive(Clone, Copy)]
    struct InternedData_<'db>(
        salsa::Id,
        std::marker::PhantomData<&'db zalsa_::interned::Value<Configuration_>>,
    );

    static INTERN_CACHE_: zalsa_::IngredientCache<
        zalsa_::interned::IngredientImpl<Configuration_>,
    > = zalsa_::IngredientCache::new();
    impl zalsa_::SalsaStructInDb for InternedData_<'_> {}
    impl zalsa_::interned::Configuration for Configuration_ {
        const DEBUG_NAME: &'static str = "Configuration";
        type Data<'db> = ();
        type Struct<'db> = InternedData_<'db>;
        fn struct_from_id<'db>(id: salsa::Id) -> Self::Struct<'db> {
            InternedData_(id, std::marker::PhantomData)
        }
        fn deref_struct(s: Self::Struct<'_>) -> salsa::Id {
            s.0
        }
    }
    impl Configuration_ {
        fn fn_ingredient(db: &dyn InternedDB) -> &zalsa_::function::IngredientImpl<Configuration_> {
            FN_CACHE_.get_or_create(db.as_dyn_database(), || {
                <dyn InternedDB as InternedDB>::zalsa_db(db);
                db.zalsa().add_or_lookup_jar_by_type(&Configuration_)
            })
        }
        fn intern_ingredient(
            db: &dyn InternedDB,
        ) -> &zalsa_::interned::IngredientImpl<Configuration_> {
            INTERN_CACHE_.get_or_create(db.as_dyn_database(), || {
                db.zalsa()
                    .add_or_lookup_jar_by_type(&Configuration_)
                    .successor(0)
            })
        }
    }
    impl zalsa_::function::Configuration for Configuration_ {
        const DEBUG_NAME: &'static str = "create_data_InternedDB";
        type DbView = dyn InternedDB;
        type SalsaStruct<'db> = InternedData_<'db>;
        type Input<'db> = ();
        type Output<'db> = InternedDBData;
        const CYCLE_STRATEGY: zalsa_::CycleRecoveryStrategy = zalsa_::CycleRecoveryStrategy::Panic;
        fn should_backdate_value(
            old_value: &Self::Output<'_>,
            new_value: &Self::Output<'_>,
        ) -> bool {
            zalsa_::should_backdate_value(old_value, new_value)
        }
        fn execute<'db>(db: &'db Self::DbView, (): ()) -> Self::Output<'db> {
            #[allow(non_snake_case)]
            fn inner_<'db>(db: &dyn InternedDB) -> InternedDBData {
                InternedDBData::new(db, None)
            }
            inner_(db)
        }
        fn recover_from_cycle<'db>(
            db: &'db dyn InternedDB,
            cycle: &zalsa_::Cycle,
            (): (),
        ) -> Self::Output<'db> {
            {
                std::mem::drop(db);
                std::mem::drop(());
                {
                    panic!("cannot recover from cycle `{0:?}`", cycle);
                }
            }
        }
        fn id_to_input<'db>(db: &'db Self::DbView, key: salsa::Id) -> Self::Input<'db> {
            Configuration_::intern_ingredient(db)
                .data(db.as_dyn_database(), key)
                .clone()
        }
    }
    impl zalsa_::Jar for Configuration_ {
        fn create_ingredients(
            &self,
            aux: &dyn zalsa_::JarAux,
            first_index: zalsa_::IngredientIndex,
        ) -> Vec<Box<dyn zalsa_::Ingredient>> {
            let fn_ingredient =
                <zalsa_::function::IngredientImpl<Configuration_>>::new(first_index, aux);
            fn_ingredient.set_capacity(0);
            <[_]>::into_vec(Box::new([
                Box::new(fn_ingredient),
                Box::new(<zalsa_::interned::IngredientImpl<Configuration_>>::new(
                    first_index.successor(0),
                )),
            ]))
        }
    }
    #[allow(non_local_definitions)]
    impl create_data_InternedDB {
        pub fn accumulated<'db, A: salsa::Accumulator>(db: &'db dyn InternedDB) -> Vec<A> {
            use salsa::plumbing as zalsa_;
            let key = Configuration_::intern_ingredient(db).intern_id(db.as_dyn_database(), ());
            Configuration_::fn_ingredient(db).accumulated_by::<A>(db, key)
        }
    }
    zalsa_::attach(db, || {
        let result = {
            let key = Configuration_::intern_ingredient(db).intern_id(db.as_dyn_database(), ());
            Configuration_::fn_ingredient(db).fetch(db, key)
        };
        <InternedDBData as std::clone::Clone>::clone(result)
    })
}
#[allow(non_camel_case_types)]
struct create_data_InternedDB {
    _priv: std::convert::Infallible,
}
trait InternedDBSetterExt: InternedDB
where
    Self: Sized,
{
    fn set_string(&mut self, __value: String) {
        use salsa::Setter;
        let data = create_data_InternedDB(self);
        data.set_string(self).to(Some(__value));
    }
    fn set_string_with_durability(&mut self, __value: String, durability: salsa::Durability) {
        use salsa::Setter;
        let data = create_data_InternedDB(self);
        data.set_string(self)
            .with_durability(durability)
            .to(Some(__value));
    }
}
impl<DB> InternedDBSetterExt for DB where DB: InternedDB {}

trait InternedDBLookupExt: InternedDB {
    fn lookup_intern_string<'db>(&self, id: InternedStringId<'_>) -> InternedString {
        id.data(self)
    }
}
impl<DB: ?Sized> InternedDBLookupExt for DB where DB: InternedDB {}

fn interned_len(db: &dyn InternedDB, id: InternedStringId<'_>) -> usize {
    let data = db.lookup_intern_string(id);
    data.data.len()
}
