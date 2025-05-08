import * as anchor from "@coral-xyz/anchor";
import {AnchorError, Program} from "@coral-xyz/anchor";
import {Vault} from "../target/types/vault";
import {assert} from "chai";

describe("sol_deposit", () => {
    const provider = anchor.AnchorProvider.env();
    anchor.setProvider(provider);

    const program = anchor.workspace.Vault as Program<Vault>;

    const user = provider.wallet;

    let vaultPda: anchor.web3.PublicKey;
    let vaultBump: number;

    const secondAccount = anchor.web3.Keypair.generate();

    before(async () => {
        [vaultPda, vaultBump] = await anchor.web3.PublicKey.findProgramAddress(
            [Buffer.from("vault"), user.publicKey.toBuffer()],
            program.programId
        );

        const sig = await provider.connection.requestAirdrop(user.publicKey, 2e9);
        await provider.connection.confirmTransaction(sig);

        const tx = await provider.connection.requestAirdrop(
            secondAccount.publicKey,
            2e9
        );
        await provider.connection.confirmTransaction(tx);
    });

    it("Initializes the vault", async () => {
        await program.methods
            .initialize()
            .rpc();

        const vault = await program.account.vault.fetch(vaultPda);
        assert.ok(vault.owner.equals(user.publicKey));
        assert.strictEqual(vault.balance.toNumber(), 0);

        await program.methods
            .initialize().accounts({
                user: secondAccount.publicKey
            }).signers([secondAccount]).rpc();
    });

    it("Deposits SOL", async () => {
        const depositAmount = new anchor.BN(1_000_000); // 0.001 SOL

        await program.methods
            .deposit(depositAmount)
            .rpc();

        const vault = await program.account.vault.fetch(vaultPda);
        assert.strictEqual(vault.balance.toNumber(), depositAmount.toNumber());
    });

    it("Withdraws SOL", async () => {
        const withdrawAmount = new anchor.BN(500_000); // 0.0005 SOL

        // check for a malicious user
        let caught = false
        try {
            await program.methods.withdraw(withdrawAmount).accounts({
                user: secondAccount.publicKey,
            }).signers([secondAccount]).rpc();
        } catch (err) {
            let e = err as AnchorError;
            assert.strictEqual(e.error.errorMessage, "Not enough funds")
            caught = true;
        }

        assert.isTrue(caught);

        await program.methods
            .withdraw(withdrawAmount)
            .rpc();

        const vault = await program.account.vault.fetch(vaultPda);
        assert.strictEqual(vault.balance.toNumber(), 500_000);
    });
});