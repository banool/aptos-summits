// Copyright (c) Aptos Labs
// SPDX-License-Identifier: Apache-2.0

module addr::summits_token {
    use addr::summits_collection::{
        is_owner as is_owner_of_collection,
    };
    use std::option;
    use std::string::{Self, String};
    use aptos_framework::chain_id::{get as get_chain_id};
    use aptos_std::object::{Self, Object};
    use aptos_std::string_utils;
    use aptos_token_objects::collection::{Self, Collection};
    use aptos_token_objects::token::{Self, MutatorRef};

    /// The caller tried to call a function that requires collection owner privileges.
    const E_CALLER_NOT_COLLECTION_OWNER: u64 = 1;

    #[resource_group_member(group = aptos_framework::object::ObjectGroup)]
    struct TokenRefs has key {
        /// We need this so the collection owner can update the URI if necessary.
        mutator_ref: MutatorRef,
    }

    /// Create a new canvas.
    public entry fun mint(
        caller: &signer,
        // The collection to create the token in.
        collection: Object<Collection>,
    ) {
        mint_(caller, collection);
    }

    // This function is separate from the top level mint function so we can use it
    // in tests. This is necessary because entry functions (correctly) cannot return
    // anything but we need it to return the object with the canvas in it. They also
    // cannot take in struct arguments, which again is convenient for testing.
    //
    // As you can see, it is a bit of a dance to mint a token where the object
    // address is used for the token URI. We should make this easier.
    //
    // TODO: This code only works if the caller is the creator of the collection
    // because you can only pass in the collection name, not Object<Collection>, to the
    // token mint functions. This means this code cannot work as minting code.
    // Because token::create only accepts a &signer, it means we can't just get the
    // collection address and pass that in too. I feel like this is bad UX.
    // As far as I can tell from the code, there is no reason the signer provided to
    // token::create to be the creator of the collection.
    //
    // In the knight example the caller is clearly only ever the collection creator.
    // Maybe it only works if I make the owner of the collection a resource / object
    // account and store a signercap. I use that to make a signer.
    //
    // https://aptos-org.slack.com/archives/C036X27DZNG/p1705852198895739
    fun mint_(
        caller: &signer,
        collection: Object<Collection>,
    ): Object<TokenRefs> {
        // TODO: Add on chain allowlist instead.
        // assert_caller_is_collection_owner(caller, collection);

        let description = string::utf8(b"todo");
        let name = string::utf8(b"todo");

        // Create the token. This mints an ObjectCore and Token.
        // TODO: Check if aggregator v2 is enabled. If so, use mint_numbered_token.
        let constructor_ref = token::create(
            caller,
            collection::name(collection),
            description,
            name,
            option::none(),
            // We use a dummy URI and then change it after once we know the object address.
            string::utf8(b"dummy"),
        );

        let mutator_ref = token::generate_mutator_ref(&constructor_ref);

        let object_signer = object::generate_signer(&constructor_ref);

        move_to(&object_signer, TokenRefs { mutator_ref });

        // It is important we call this after we moved something into the container
        // first before calling this, otherwise there will be no ObjectCore there yet.
        // TODO: This doesn't make sense to me but it seems to be working that way.
        let obj = object::object_from_constructor_ref(&constructor_ref);

        // See https://aptos-org.slack.com/archives/C03N9HNSUB1/p1686764312687349 for
        // more info on this mess. Trim the the leading @.
        let object_address = object::object_address(&obj);
        let object_address_string = string_utils::to_string_with_canonical_addresses(&object_address);
        let object_address_string = string::sub_string(
            &object_address_string,
            1,
            string::length(&object_address_string),
        );
        let chain_id = get_chain_id();
        let network_str = if (chain_id == 1) {
            b"mainnet"
        } else if (chain_id == 2) {
            b"testnet"
        } else if (chain_id == 4) {
            b"localnet"
        } else {
            b"devnet"
        };
        let uri = string::utf8(b"https://api.summits.dport.me/");
        string::append(&mut uri, string::utf8(network_str));
        string::append(&mut uri, string::utf8(b"/media/0x"));
        string::append(&mut uri, object_address_string);
        // TODO: This might end up being a GIF.
        string::append(&mut uri, string::utf8(b".png"));

        // Set the real URI.
        token::set_uri(&token::generate_mutator_ref(&constructor_ref), uri);

        obj
    }

    ///////////////////////////////////////////////////////////////////////////////////
    //                                  Collection owner                             //
    ///////////////////////////////////////////////////////////////////////////////////

    fun assert_caller_is_collection_owner(caller: &signer, collection: Object<Collection>) {
        assert!(is_owner_of_collection(caller, collection), E_CALLER_NOT_COLLECTION_OWNER);
    }

    ///////////////////////////////////////////////////////////////////////////////////
    //                                 Collection owner                              //
    ///////////////////////////////////////////////////////////////////////////////////
    // Functions that only the collection owner can call.

    /// Set the URI for the token. This is necessary if down the line we change how we
    /// generate the image.
    public entry fun set_uri(caller: &signer, refs: Object<TokenRefs>, uri: String) acquires TokenRefs {
        let collection = token::collection_object(refs);
        assert_caller_is_collection_owner(caller, collection);
        let object_addr = object::object_address(&refs);
        let refs_ = borrow_global<TokenRefs>(object_addr);
        token::set_uri(&refs_.mutator_ref, uri);
    }

    ///////////////////////////////////////////////////////////////////////////////////
    //                                     Tests                                     //
    ///////////////////////////////////////////////////////////////////////////////////

    #[test_only]
    use addr::summits_collection::create_for_test as create_collection_for_test;
    #[test_only]
    use std::timestamp;
    #[test_only]
    use aptos_framework::aptos_coin::{Self, AptosCoin};
    #[test_only]
    use aptos_framework::account::{Self};
    #[test_only]
    use aptos_framework::coin;
    #[test_only]
    use aptos_framework::chain_id;
    #[test_only]
    use std::signer;

    #[test_only]
    const ONE_APT: u64 = 100000000;

    #[test_only]
    const STARTING_BALANCE: u64 = 50 * 100000000;

    #[test_only]
    /// Create a test account with some funds.
    fun mint_test_account(
        _caller: &signer,
        aptos_framework: &signer,
        account: &signer,
    ) {
        if (!aptos_coin::has_mint_capability(aptos_framework)) {
            // If aptos_framework doesn't have the mint cap it means we need to do some
            // initialization. This function will initialize AptosCoin and store the
            // mint cap in aptos_framwork. These capabilities that are returned from the
            // function are just copies. We don't need them since we use aptos_coin::mint
            // to mint coins, which uses the mint cap from the MintCapStore on
            // aptos_framework. So we burn them.
            let (burn_cap, mint_cap) = aptos_coin::initialize_for_test(aptos_framework);
            coin::destroy_burn_cap(burn_cap);
            coin::destroy_mint_cap(mint_cap);
        };
        account::create_account_for_test(signer::address_of(account));
        coin::register<AptosCoin>(account);
        aptos_coin::mint(aptos_framework, signer::address_of(account), STARTING_BALANCE);
    }

    #[test_only]
    public fun set_global_time(
        aptos_framework: &signer,
        timestamp: u64
    ) {
        timestamp::set_time_has_started_for_testing(aptos_framework);
        timestamp::update_global_time_for_test_secs(timestamp);
    }

    #[test_only]
    fun init_test(caller: &signer, friend1: &signer, friend2: &signer, aptos_framework: &signer) {
        set_global_time(aptos_framework, 100);
        chain_id::initialize_for_test(aptos_framework, 3);
        create_collection_for_test(caller);
        mint_test_account(caller, aptos_framework, caller);
        mint_test_account(caller, aptos_framework, friend1);
        mint_test_account(caller, aptos_framework, friend2);
    }

    #[test_only]
    fun get_collection(): Object<Collection> {
        let name = string::utf8(b"test");
        let collection_address = collection::create_collection_address(&@addr, &name);
        object::address_to_object<Collection>(collection_address)
    }

    #[test_only]
    fun mint_token(
        caller: &signer,
    ): Object<TokenRefs> {
        mint_(caller, get_collection())
    }

    #[test(caller = @addr, friend1 = @0x456, friend2 = @0x789, aptos_framework = @aptos_framework)]
    fun test_mint(caller: signer, friend1: signer, friend2: signer, aptos_framework: signer) {
        init_test(&caller, &friend1, &friend2, &aptos_framework);
        // This only works if the caller is `caller`. See the comment above mint_.
        let tok = mint_token(&caller);
        aptos_std::debug::print(&token::uri(tok));
    }
}