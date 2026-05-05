// @generated automatically by Diesel CLI.

diesel::table! {
    use diesel::sql_types::*;
    use diesel_async::*;

    buyers (id) {
        id -> Text,
        wallet_id -> Text,
        wallet_address -> Text,
        public_key -> Text,
        network -> Text,
        created_at -> Timestamptz,
    }
}

diesel::table! {
    use diesel::sql_types::*;
    use diesel_async::*;

    merchants (id) {
        id -> Text,
        email -> Text,
        business_name -> Text,
        wallet_id -> Text,
        wallet_address -> Text,
        public_key -> Text,
        currency -> Varchar,
        country -> Varchar,
        network -> Text,
        logo_url -> Nullable<Text>,
        is_active -> Bool,
        created_at -> Timestamptz,
        updated_at -> Timestamptz,
    }
}

diesel::table! {
    use diesel::sql_types::*;
    use diesel_async::*;

    transactions (id) {
        id -> Text,
        merchant_id -> Text,
        tx_hash -> Text,
        from_address -> Text,
        to_address -> Text,
        amount -> Numeric,
        currency -> Varchar,
        status -> Text,
        block_number -> Nullable<Text>,
        note -> Nullable<Text>,
        email_sent -> Bool,
        detected_at -> Timestamptz,
        confirmed_at -> Nullable<Timestamptz>,
    }
}

diesel::joinable!(transactions -> merchants (merchant_id));

diesel::allow_tables_to_appear_in_same_query!(buyers, merchants, transactions);
