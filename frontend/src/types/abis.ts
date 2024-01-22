// See the README for an explanation of where these came from.

export const COLLECTION_ABI = {
  "address": "0x296102a3893d43e11de2aa142fbb126377120d7d71c246a2f95d5b4f3ba16b30",
  "name": "summits_collection",
  "friends": [],
  "exposed_functions": [
    {
      "name": "create",
      "visibility": "public",
      "is_entry": true,
      "is_view": false,
      "generic_type_params": [],
      "params": [
        "&signer",
        "0x1::string::String",
        "u64",
        "bool"
      ],
      "return": []
    },
    {
      "name": "is_owner",
      "visibility": "public",
      "is_entry": false,
      "is_view": false,
      "generic_type_params": [],
      "params": [
        "&signer",
        "0x1::object::Object<0x4::collection::Collection>"
      ],
      "return": [
        "bool"
      ]
    },
    {
      "name": "set_description",
      "visibility": "public",
      "is_entry": true,
      "is_view": false,
      "generic_type_params": [],
      "params": [
        "&signer",
        "0x1::object::Object<0x4::collection::Collection>",
        "0x1::string::String"
      ],
      "return": []
    },
    {
      "name": "set_uri",
      "visibility": "public",
      "is_entry": true,
      "is_view": false,
      "generic_type_params": [],
      "params": [
        "&signer",
        "0x1::object::Object<0x4::collection::Collection>",
        "0x1::string::String"
      ],
      "return": []
    },
    {
      "name": "transfer",
      "visibility": "public",
      "is_entry": true,
      "is_view": false,
      "generic_type_params": [],
      "params": [
        "&signer",
        "0x1::object::Object<0x4::collection::Collection>",
        "address"
      ],
      "return": []
    }
  ],
  "structs": [
    {
      "name": "CollectionRefs",
      "is_native": false,
      "abilities": [
        "key"
      ],
      "generic_type_params": [],
      "fields": [
        {
          "name": "transfer_ref",
          "type": "0x1::object::TransferRef"
        },
        {
          "name": "mutator_ref",
          "type": "0x4::collection::MutatorRef"
        }
      ]
    }
  ]
} as const;

export const TOKEN_ABI = {
  "address": "0x296102a3893d43e11de2aa142fbb126377120d7d71c246a2f95d5b4f3ba16b30",
  "name": "summits_token",
  "friends": [],
  "exposed_functions": [
    {
      "name": "mint",
      "visibility": "public",
      "is_entry": true,
      "is_view": false,
      "generic_type_params": [],
      "params": [
        "&signer",
        "0x1::object::Object<0x4::collection::Collection>"
      ],
      "return": []
    },
    {
      "name": "set_uri",
      "visibility": "public",
      "is_entry": true,
      "is_view": false,
      "generic_type_params": [],
      "params": [
        "&signer",
        "0x1::object::Object<0x296102a3893d43e11de2aa142fbb126377120d7d71c246a2f95d5b4f3ba16b30::summits_token::TokenRefs>",
        "0x1::string::String"
      ],
      "return": []
    }
  ],
  "structs": [
    {
      "name": "TokenRefs",
      "is_native": false,
      "abilities": [
        "key"
      ],
      "generic_type_params": [],
      "fields": [
        {
          "name": "mutator_ref",
          "type": "0x4::token::MutatorRef"
        }
      ]
    }
  ]
} as const;

export const GENR_ABI = {
  "address": "0x296102a3893d43e11de2aa142fbb126377120d7d71c246a2f95d5b4f3ba16b30",
  "name": "genr",
  "friends": [],
  "exposed_functions": [
    {
      "name": "create_code_reference",
      "visibility": "public",
      "is_entry": false,
      "is_view": false,
      "generic_type_params": [],
      "params": [
        "&0x1::object::ConstructorRef",
        "0x1::string::String",
        "bool"
      ],
      "return": []
    },
    {
      "name": "create_empty_holder",
      "visibility": "public",
      "is_entry": false,
      "is_view": false,
      "generic_type_params": [],
      "params": [
        "&0x1::object::ConstructorRef",
        "bool"
      ],
      "return": []
    },
    {
      "name": "update_code_holder",
      "visibility": "public",
      "is_entry": true,
      "is_view": false,
      "generic_type_params": [],
      "params": [
        "&signer",
        "0x1::object::Object<0x296102a3893d43e11de2aa142fbb126377120d7d71c246a2f95d5b4f3ba16b30::genr::CodeHolder>",
        "vector<u8>",
        "u8"
      ],
      "return": []
    },
    {
      "name": "update_code_reference",
      "visibility": "public",
      "is_entry": true,
      "is_view": false,
      "generic_type_params": [],
      "params": [
        "&signer",
        "0x1::object::Object<0x296102a3893d43e11de2aa142fbb126377120d7d71c246a2f95d5b4f3ba16b30::genr::CodeReference>",
        "0x1::string::String"
      ],
      "return": []
    }
  ],
  "structs": [
    {
      "name": "CodeHolder",
      "is_native": false,
      "abilities": [
        "store",
        "key"
      ],
      "generic_type_params": [],
      "fields": [
        {
          "name": "code",
          "type": "vector<u8>"
        },
        {
          "name": "format",
          "type": "u8"
        },
        {
          "name": "allow_updates",
          "type": "bool"
        }
      ]
    },
    {
      "name": "CodeReference",
      "is_native": false,
      "abilities": [
        "store",
        "key"
      ],
      "generic_type_params": [],
      "fields": [
        {
          "name": "url",
          "type": "0x1::string::String"
        },
        {
          "name": "allow_updates",
          "type": "bool"
        }
      ]
    }
  ]
} as const;
