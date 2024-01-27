// Copyright (c) Aptos Labs
// SPDX-License-Identifier: Apache-2.0

//! See the README for more information about how this module works.

/// A note on the set_* functions:
/// This module allows the owner of the collection to transfer ownership to another
/// account. As such, in order to determine the current owner of the collection, we
/// must use the collection creator and collection name to determine its address and
/// then check the owner that way, rather than just assume the creator is the owner.

module addr::summits_collection {
    use std::error;
    use std::option;
    use std::signer;
    use std::string::{Self, String};
    // use addr::genr::create_empty_holder;
    use aptos_std::object::{Self, Object, ExtendRef};
    use aptos_token_objects::collection::{Self, Collection, MutatorRef};

    friend addr::summits_token;

    /// The account trying to create the collection is not allowed to do so.
    const E_COLLECTION_CREATOR_FORBIDDEN: u64 = 1;

    /// The account trying to mutate the collection is not allowed to do so.
    const E_COLLECTION_MUTATOR_FORBIDDEN: u64 = 2;

    /// The account trying to transfer ownership of the collection is not allowed to do so.
    const E_COLLECTION_TRANSFERER_FORBIDDEN: u64 = 3;

    /// The account that is allowed to create the collection. For now we just enforce
    /// that the collection creator is the same account that published the module.
    const PERMITTED_COLLECTION_CREATOR: address = @addr;

    /// Resource we store at the object address to enable mutation and transfer
    /// ownership of the collection.
    struct CollectionRefs has key {
        collection_mutator_ref: MutatorRef,
        owner_extend_ref: ExtendRef,
    }

    const COLLECTION_NAME: vector<u8> = b"Aptos Passport: Summit One";
    const COLLECTION_SEED: vector<u8> = b"AptosPassportSummitOneOwnerSeed";

    // NOTE: This has been changed to be specific to the summits collection. You can
    // only call this once courtesy of the hardcoded SEED. It is just simpler this way.
    // As such the code is a bit of a mess, in some cases you can pass in args for
    // creating the collection even though you can only call this function once.
    // Clean up required, though I disagree with the token API as it is now. We'll keep
    // discussing it.
    public entry fun create(publisher: &signer) {
        // For now only allow the module publisher to create collections.
        assert!(
            signer::address_of(publisher) == PERMITTED_COLLECTION_CREATOR,
            error::invalid_argument(E_COLLECTION_CREATOR_FORBIDDEN),
        );

        let name = string::utf8(COLLECTION_NAME);
        let max_supply = 100;

        // Create an object that will own the collection. This is necessary due to
        // intentional restrictiveness in our token API.
        // https://aptos-org.slack.com/archives/C036X27DZNG/p1705852198895739
        let constructor_ref = object::create_named_object(publisher, COLLECTION_SEED);
        let collection_owner_signer = object::generate_signer(&constructor_ref);

        // Generate an extend ref so we can get a signer to mint tokens in the collection.
        let owner_extend_ref = object::generate_extend_ref(&constructor_ref);

        let constructor_ref = collection::create_fixed_collection(
            &collection_owner_signer,
            string::utf8(b"Stamps to memorialize the first ever Aptos Ecosystem Summit."),
            max_supply,
            name,
            option::none(),
            // We just use one of the tokens.
            string::utf8(b"https://storage.googleapis.com/aptos-summits/images/0xc0881d4b59a54bbd5c0015a3c42ee10bc3ee824776b021a1636297f27552f0a4.png"),
        );

        let collection_mutator_ref = collection::generate_mutator_ref(&constructor_ref);
        let collection_refs = CollectionRefs {
            collection_mutator_ref,
            owner_extend_ref
        };

        // TODO: For now the code is just baked into the frontend.
        // Create a holder for the code.
        // create_empty_holder(&constructor_ref, allow_code_updates);

        let object_signer = object::generate_signer(&constructor_ref);

        // Store the refs alongside the collection.
        move_to(&object_signer, collection_refs);
    }

    #[test_only]
    public fun create_for_test(publisher: &signer) {
        create(publisher);
    }

    /// Set the URI of the collection.
    public entry fun set_uri(caller: &signer, uri: String) acquires CollectionRefs {
        assert!(
            is_creator(caller),
            error::invalid_argument(E_COLLECTION_MUTATOR_FORBIDDEN),
        );
        let collection = get_collection();
        let collection_refs = borrow_global<CollectionRefs>(object::object_address(&collection));
        collection::set_uri(&collection_refs.collection_mutator_ref, uri);
    }

    /// Set the description of the collection.
    public entry fun set_description(caller: &signer, description: String) acquires CollectionRefs {
        assert!(
            is_creator(caller),
            error::invalid_argument(E_COLLECTION_MUTATOR_FORBIDDEN),
        );
        let collection = get_collection();
        let collection_refs = borrow_global<CollectionRefs>(object::object_address(&collection));
        collection::set_description(&collection_refs.collection_mutator_ref, description);
    }

    /// Get the collection. Note, if the module is republished with a different
    /// address for the permitted collection creator after the collection has been
    /// created, this will cease to work. Same thing if the collection name is changed.
    public fun get_collection(): Object<Collection> {
        // Get the address of the account we created to own the collection.
        let collection_creator_address = object::create_object_address(
            &PERMITTED_COLLECTION_CREATOR,
            COLLECTION_SEED,
        );
        // Pass that in to figure out the collection address.
        let collection_address = collection::create_collection_address(
            &collection_creator_address,
            &string::utf8(COLLECTION_NAME),
        );
        object::address_to_object<Collection>(collection_address)
    }

    // This is not is_owner, it is based on where the contract is deployed, not who
    // owns the collection.
    public fun is_creator(caller: &signer): bool {
        signer::address_of(caller) == PERMITTED_COLLECTION_CREATOR
    }

    /*
    public fun is_owner(caller: &signer): bool {
        let collection = get_collection();
        object::is_owner<Collection>(collection, signer::address_of(caller))
    }
    */

    // So we can mint tokens in the collection.
    public(friend) fun get_collection_owner_signer(): signer acquires CollectionRefs {
        let collection = get_collection();
        let collection_refs = borrow_global<CollectionRefs>(object::object_address(&collection));
        object::generate_signer_for_extending(&collection_refs.owner_extend_ref)
    }

    public fun get_collection_name(): String {
        string::utf8(COLLECTION_NAME)
    }
}
