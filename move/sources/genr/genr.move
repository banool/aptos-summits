// Copyright (c) Aptos Labs
// SPDX-License-Identifier: Apache-2.0

// TODO: Think of a better name for this.
// TODO: Move this into a separate location.
// TODO: For this to be generic there has to be an ABI / API for how to run and provide
// input to the code.

module addr::genr {
    use std::error;
    use std::signer;
    use std::vector;
    use aptos_std::object::{Self, ConstructorRef, Object};
    use aptos_std::string::String;

    const FORMAT_WEBGL: u8 = 1;

    /// You are not the owner of the collection, you cannot update the code.
    const E_NOT_COLLECTION_OWNER: u64 = 1;
    /// The CodeHolder was configured to not allowed updating the code.
    const E_UPDATES_NOT_ALLOWED: u64 = 2;

    // For simplicity we just determine if the holder is empty by if the vector
    // is empty.
    // TODO: Make a version where there is a URL to code off chain. A rust like enum
    // type would be very handy for this.
    struct CodeHolder has key, store {
        /// Code in the specified format.
        code: vector<u8>,
        // TODO: A public enum would be great for this.
        format: u8,
        /// Whether it should be allowed to update the code after the initial upload.
        allow_updates: bool,
    }

    /// Create a CodeHolder at an object.
    public fun create_empty_holder(
        constructor_ref: &ConstructorRef,
        allow_updates: bool
    ) {
        let object_signer = object::generate_signer(constructor_ref);
        move_to(
            &object_signer,
            CodeHolder {
                code: vector::empty(),
                format: 0,
                allow_updates,
            },
        );
    }

    public entry fun update_code_holder(
        caller: &signer,
        holder: Object<CodeHolder>,
        code: vector<u8>,
        format: u8
    )
        acquires CodeHolder {
        assert !(
            object::is_owner<CodeHolder>(holder, signer::address_of(caller)),
            error::invalid_state(E_NOT_COLLECTION_OWNER),
        );
        let holder_ = borrow_global_mut<CodeHolder>(object::object_address(&holder));
        assert !(
            holder_.allow_updates && vector::is_empty(&holder_.code),
            error::invalid_state(E_UPDATES_NOT_ALLOWED),
        );
        holder_.code = code;
        holder_.format = format;
    }

    struct CodeReference has key, store {
        /// Code in the specified format.
        url: String,
        /// Whether it should be allowed to update the URL after initial creation.
        allow_updates: bool,
    }

    /// Create a CodeHolder at an object.
    public fun create_code_reference(
        constructor_ref: &ConstructorRef,
        url: String,
        allow_updates: bool
    ) {
        let object_signer = object::generate_signer(constructor_ref);
        move_to(
            &object_signer,
            CodeReference {url, allow_updates,},
        );
    }

    public entry fun update_code_reference(
        caller: &signer,
        reference: Object<CodeReference>,
        url: String
    )
        acquires CodeReference {
        assert !(
            object::is_owner<CodeReference>(
                reference,
                signer::address_of(caller)
            ),
            error::invalid_state(E_NOT_COLLECTION_OWNER),
        );
        let reference_ = borrow_global_mut<CodeReference>(
            object::object_address(&reference)
        );
        assert !(
            reference_.allow_updates,
            error::invalid_state(E_UPDATES_NOT_ALLOWED),
        );
        reference_.url = url;
    }
}
