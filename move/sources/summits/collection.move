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
    use addr::genr::create_empty_holder;
    use aptos_std::object::{Self, Object, TransferRef};
    use aptos_token_objects::collection::{Self, Collection, MutatorRef};

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
        transfer_ref: TransferRef,
        mutator_ref: MutatorRef,
    }

    /// To maximize the code size, we let the collection creator upload / update the
    /// code in a separate txn.
    public entry fun create(publisher: &signer, name: string::String, max_supply: u64, allow_code_updates: bool) {
        // For now only allow the module publisher to create collections.
        assert!(
            signer::address_of(publisher) == PERMITTED_COLLECTION_CREATOR,
            error::invalid_argument(E_COLLECTION_CREATOR_FORBIDDEN),
        );
        let constructor_ref = collection::create_fixed_collection(
            publisher,
            string::utf8(b"unset"),
            max_supply,
            name,
            option::none(),
            string::utf8(b"unset"),
        );
        let transfer_ref = object::generate_transfer_ref(&constructor_ref);
        let mutator_ref = collection::generate_mutator_ref(&constructor_ref);
        let collection_refs = CollectionRefs {
            transfer_ref,
            mutator_ref,
        };

        // Create a holder for the code.
        create_empty_holder(&constructor_ref, allow_code_updates);

        let object_signer = object::generate_signer(&constructor_ref);

        // Store the collection refs.
        move_to(&object_signer, collection_refs);
    }

    #[test_only]
    public fun create_for_test(publisher: &signer) {
        create(publisher, string::utf8(b"test"), 10, true);
    }

    /// Set the URI of the collection.
    public entry fun set_uri(caller: &signer, collection: Object<Collection>, uri: String) acquires CollectionRefs {
        assert!(
            is_owner(caller, collection),
            error::invalid_argument(E_COLLECTION_MUTATOR_FORBIDDEN),
        );
        let collection_refs = borrow_global<CollectionRefs>(object::object_address(&collection));
        collection::set_uri(&collection_refs.mutator_ref, uri);
    }

    /// Set the description of the collection.
    public entry fun set_description(caller: &signer, collection: Object<Collection>, description: String) acquires CollectionRefs {
        assert!(
            is_owner(caller, collection),
            error::invalid_argument(E_COLLECTION_MUTATOR_FORBIDDEN),
        );
        let collection_refs = borrow_global<CollectionRefs>(object::object_address(&collection));
        collection::set_description(&collection_refs.mutator_ref, description);
    }

    /// Transfer ownership of the collection.
    public entry fun transfer(caller: &signer, collection: Object<Collection>, to: address) acquires CollectionRefs {
        assert!(
            is_owner(caller, collection),
            error::invalid_argument(E_COLLECTION_TRANSFERER_FORBIDDEN),
        );
        let collection_refs = borrow_global<CollectionRefs>(object::object_address(&collection));
        let linear_transfer_ref = object::generate_linear_transfer_ref(&collection_refs.transfer_ref);
        object::transfer_with_ref(linear_transfer_ref, to);
    }

    public fun is_owner(caller: &signer, collection: Object<Collection>): bool {
        object::is_owner<Collection>(collection, signer::address_of(caller))
    }
}
