CREATE TABLE addresses (
    id SERIAL PRIMARY KEY,
    address TEXT NOT NULL UNIQUE,
    created_at TIMESTAMP WITH TIME ZONE DEFAULT NOW()
);
CREATE TABLE pubkeys (
    id SERIAL PRIMARY KEY,
    pubkey TEXT NOT NULL UNIQUE,
    created_at TIMESTAMP WITH TIME ZONE DEFAULT NOW()
);
CREATE TABLE addresses_pubkeys (
    address_id INTEGER NOT NULL REFERENCES addresses (id),
    pubkey_id INTEGER NOT NULL REFERENCES pubkeys (id),
    created_at TIMESTAMP WITH TIME ZONE DEFAULT NOW(),
    PRIMARY KEY(address_id, pubkey_id)
);
