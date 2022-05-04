localnet-validator:
	solana-test-validator

localnet-logs:
	solana logs --url localhost

localnet-init:
	solana config set --url localhost
	solana-keygen new -o keypair.json
	solana airdrop 10 keypair.json -u localhost

localnet-deploy:
	cd program; cargo build-bpf --manifest-path=Cargo.toml --bpf-out-dir=dist/program
	solana deploy --keypair keypair.json dist/program/donation_lamports.so -u localhost


