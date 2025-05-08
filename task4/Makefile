.PHONY: test
test:
	cargo build --manifest-path ../task4_client/Cargo.toml
	cp ../task4_client/target/debug/task4_client vault_cli

	solana airdrop -u http://127.0.0.1:8899 -k ./payer.json 5
	export ANCHOR_WALLET=./payer.json
	export ANCHOR_PROVIDER_URL=http://127.0.0.1:8899
	anchor deploy -- --commitment confirmed
	sleep 10
	./vault_cli ./config.yaml initialize
	./vault_cli ./config.yaml balance
	./vault_cli ./config.yaml deposit 100
	./vault_cli ./config.yaml balance
	./vault_cli ./config.yaml withdraw 50
	./vault_cli ./config.yaml balance
	rm ./vault_cli