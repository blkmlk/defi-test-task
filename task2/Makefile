.PHONY: test
test:
	solana airdrop -u http://127.0.0.1:8899 -k ./sender1.json 5
	solana airdrop -u http://127.0.0.1:8899 -k ./sender2.json 5
	cargo run -- config.yaml