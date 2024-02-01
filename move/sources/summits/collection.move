// Copyright (c) Aptos Labs
// SPDX-License-Identifier: Apache-2.0

module addr::summits_collection {
    use std::error;
    use std::option;
    use std::signer;
    use std::string::{Self, String};
    use aptos_std::object::{Self, Object, ExtendRef};
    use aptos_std::smart_table::{Self, SmartTable};
    use aptos_token_objects::collection::{Self, Collection, MutatorRef};

    friend addr::summits_token;

    /// The caller tried to call a function that requires collection owner privileges.
    const E_CALLER_NOT_COLLECTION_OWNER: u64 = 1;

    /// The account that is allowed to create the collection. For now we just enforce
    /// that the collection creator is the same account that published the module.
    const PERMITTED_COLLECTION_CREATOR: address = @addr;

    /// Resource we store at the object address to enable mutation and transfer
    /// ownership of the collection.
    struct CollectionRefs has key {
        collection_mutator_ref: MutatorRef,
        owner_extend_ref: ExtendRef,
    }

    /// Track who we have minted a token in the collection to.
    struct TokenOwners has key {
        owners: SmartTable<address, bool>,
    }

    const COLLECTION_NAME: vector<u8> = b"APTOS PASSPORT: Ecosystem Summit One";
    const COLLECTION_SEED: vector<u8> = b"AptosPassportEcosystemSummitOneSeed";

    /// Create the collection and all the related structs.
    /// You can only call this once unless you change COLLECTION_SEED.
    public entry fun create(publisher: &signer) {
        // For now only allow the module publisher to create collections.
        assert_caller_is_collection_creator(publisher);

        let name = get_collection_name();
        let max_supply = 250;

        // Create an object that will own the collection. This is necessary due to
        // intentional restrictiveness in our token API.
        // https://aptos-org.slack.com/archives/C036X27DZNG/p1705852198895739
        let constructor_ref = object::create_named_object(publisher, COLLECTION_SEED);
        let collection_owner_signer = object::generate_signer(&constructor_ref);

        // Generate an extend ref so we can get a signer to mint tokens in the collection.
        let owner_extend_ref = object::generate_extend_ref(&constructor_ref);

        let constructor_ref = collection::create_fixed_collection(
            &collection_owner_signer,
            // \xF0\x9F\x8C\x90 is the globe emoji.
            string::utf8(
                b"This NFT collection commemorates the first ever Aptos Ecosystem Summit from January 22-26, 2024. This week brought together 40+ premier Aptos projects, partners, and supporters to celebrate Aptos innovation across the ecosystem. These NFTs are soulbound to honor the growing community of builders who have gathered for Aptos in real life. The artwork is algorithmically generated, so every piece of art is completely unique. As part of the APTOS PASSPORT, these NFTs will serve as an access point for deeper connection with the Aptos community. Thank you to everyone who joined the Aptos Foundation for the 2024 Aptos Ecosystem Summit in Palo Alto, CA. Make Every M\xF0\x9F\x8C\x90ve Count."
            ),
            max_supply,
            name,
            option::none(),
            // We just use one of the tokens.
            string::utf8(
                b"https://storage.googleapis.com/aptos-summits/images/collection.png"
            ),
        );

        let collection_mutator_ref = collection::generate_mutator_ref(&constructor_ref);
        let collection_refs = CollectionRefs {
            collection_mutator_ref,
            owner_extend_ref
        };

        // TODO: For now the code is just baked into the frontend, it is not on chain
        // or on Arweave or anything.
        // Create a holder for the code.
        // create_empty_holder(&constructor_ref, allow_code_updates);

        let object_signer = object::generate_signer(&constructor_ref);

        // Store the refs alongside the collection.
        move_to(&object_signer, collection_refs);

        // Store the map of who owns a token in the collection.
        move_to(
            &object_signer,
            TokenOwners {owners: smart_table::new(),},
        );

    }

    #[test_only]
    public fun create_for_test(publisher: &signer) {
        create(publisher);
    }

    /// Set the URI of the collection.
    public entry fun set_uri(caller: &signer, uri: String)
        acquires CollectionRefs {
        assert_caller_is_collection_creator(caller);
        let collection = get_collection();
        let collection_refs = borrow_global<CollectionRefs>(
            object::object_address(&collection)
        );
        collection::set_uri(
            &collection_refs.collection_mutator_ref,
            uri
        );
    }

    /// Set the description of the collection.
    public entry fun set_description(caller: &signer, description: String)
        acquires CollectionRefs {
        assert_caller_is_collection_creator(caller);
        let collection = get_collection();
        let collection_refs = borrow_global<CollectionRefs>(
            object::object_address(&collection)
        );
        collection::set_description(
            &collection_refs.collection_mutator_ref,
            description
        );
    }

    /// Get the collection. Note, if the module is republished with a different
    /// address for the permitted collection creator after the collection has been
    /// created, this will cease to work. Same thing if the collection name is changed.
    public(friend) fun get_collection(): Object<Collection>{
        // Get the address of the account we created to own the collection.
        let collection_creator_address = object::create_object_address(
            &PERMITTED_COLLECTION_CREATOR,
            COLLECTION_SEED,
        );
        // Pass that in to figure out the collection address.
        let collection_address = collection::create_collection_address(
            &collection_creator_address,
            &get_collection_name(),
        );
        object::address_to_object<Collection>(collection_address)
    }

    /// So we can mint tokens in the collection. Friend function so only token.move can
    /// call it.
    public(friend) fun get_collection_owner_signer(): signer
        acquires CollectionRefs {
        let collection = get_collection();
        let collection_refs = borrow_global<CollectionRefs>(
            object::object_address(&collection)
        );
        object::generate_signer_for_extending(&collection_refs.owner_extend_ref)
    }

    public fun get_collection_name(): String {
        string::utf8(COLLECTION_NAME)
    }

    /// This is not is_owner, it is based on where the contract is deployed, not who
    /// owns the collection. The contract deployer is the one we give privileges to.
    public fun is_creator(caller: &signer): bool {
        signer::address_of(caller) == PERMITTED_COLLECTION_CREATOR
    }

    /// Confirm the caller is the creator of the collection. Notably they're not the
    /// owner, an object that the caller owns is.
    public fun assert_caller_is_collection_creator(caller: &signer) {
        assert !(
            is_creator(caller),
            error::invalid_state(E_CALLER_NOT_COLLECTION_OWNER)
        );
    }

    /// Returns true if the given account owns a token in the collection.
    public fun is_token_owner(address: address): bool
        acquires TokenOwners {
        let collection = get_collection();
        let token_owners = borrow_global<TokenOwners>(
            object::object_address(&collection)
        );
        smart_table::contains(&token_owners.owners, address)
    }

    /// Record that we minted a token in the collection to the given address.
    public(friend) fun record_minted(address: address)
        acquires TokenOwners {
        let collection = get_collection();
        let token_owners = borrow_global_mut<TokenOwners>(
            object::object_address(&collection)
        );
        smart_table::add(
            &mut token_owners.owners,
            address,
            true
        );
    }
}
