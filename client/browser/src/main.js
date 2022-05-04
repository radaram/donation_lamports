import Wallet from "@project-serum/sol-wallet-adapter";
import {
    Connection,
    SystemProgram,
    Transaction,
    PublicKey,
    TransactionInstruction,
    Keypair
} from "@solana/web3.js";
import { deserialize, serialize } from "borsh";

const cluster = "https://api.devnet.solana.com";
//const cluster = "http://localhost:8899";
const connection = new Connection(cluster, "confirmed");
const programId = new PublicKey(
    "9nhEXCXcd5BdYyqya5ovZhMrLWbRTXa8de9EnmFUPEcv"
);
const wallet = new Wallet("https://www.sollet.io", cluster);

wallet.on("connect", (publicKey) => console.log("sollet connected", publicKey.toBase58()))

let organizationPubkey;


export async function onDonate() {
    let amount = document.getElementById("amount").value;
    console.log("Donate amount: " + amount);
    if (!amount || amount < 0) {
        alert("amount is not valid");
        return
    }
   if (!wallet.connected) {
        await wallet.connect()
    }
 
    await donate(
        wallet.publicKey,
        amount
    );
}


export async function onGetAllDonations() {
   if (!wallet.connected) {
        await wallet.connect()
    }
 
    let donations = await getAllDonations()
    let donationsElem = document.getElementById("donations")
    donationsElem.innerHTML = ""
    donationsElem.innerHTML += "<ul>"
    donations.forEach((item) => {
        donationsElem.innerHTML += "<li>" + item.userPubKey.toString() + " " + item.amount + "</li>"    
    })

    donationsElem.innerHTML += "</ul>"
}


export async function onWithdraw() {
   if (!wallet.connected) {
        await wallet.connect()
    }
 
    await withdraw()
}


class DonateDetails {
    constructor(properties) {
        Object.keys(properties).forEach((key) => {
            this[key] = properties[key];
        });
    }
    static schema = new Map([[DonateDetails,
        {
            kind: "struct",
            fields: [
                ["user", [32]],
                ["amount", "u64"],
            ]
        }]]);

}


async function prepareTransaction(userPubKey, amount) { 
  const SEED = "crypton" + Math.random().toString();
  let donatorPublicKey = await PublicKey.createWithSeed(
        wallet.publicKey,
        SEED,
        programId
  );
    
  let donateDetails = new DonateDetails({
      user: wallet.publicKey.toBuffer(),
      amount: amount, 
  });
  let data = serialize(DonateDetails.schema, donateDetails);
  let data_to_send = new Uint8Array([1, ...data]);
 
  const lamports = (await connection.getMinimumBalanceForRentExemption(data.length));

  const donatorProgramAccount = SystemProgram.createAccountWithSeed({
        fromPubkey: wallet.publicKey,
        basePubkey: wallet.publicKey,
        seed: SEED,
        newAccountPubkey: donatorPublicKey,
        lamports: lamports,
        space: data.length,
        programId: programId,
  });

  const instruction = new TransactionInstruction({
    keys: [
      { pubkey: userPubKey, isSigner: true, isWritable: true },
      { pubkey: donatorPublicKey, isSigner: false, isWritable: true },
      { pubkey: SystemProgram.programId, isSigner: false, isWritable: false },
    ],
    programId: programId,
    data: data_to_send,
  })

  console.log("donate details", donateDetails)
  
  let tx = new Transaction()
  tx.add(donatorProgramAccount)
  tx.add(instruction)

  tx.feePayer = userPubKey
  tx.recentBlockhash = (await connection.getRecentBlockhash()).blockhash

  return tx
}


async function donate(userPubKey, amount) {
  console.log("donate called")
  const tx = await prepareTransaction(userPubKey, amount)
  let signed = await wallet.signTransaction(tx)
  await broadcastSignedTransaction(signed)
}

async function broadcastSignedTransaction(signed) {
  let signature = await connection.sendRawTransaction(signed.serialize())
  console.log("Submitted transaction " + signature + ", awaiting confirmation")
  await connection.confirmTransaction(signature)
  console.log("Transaction " + signature + " confirmed")
}


async function getAllDonations() {
    let accounts = await connection.getProgramAccounts(programId);
    let result = [];
    let donate_details;
    accounts.forEach((item) => {
        try {
            donate_details = deserialize(DonateDetails.schema, DonateDetails, item.account.data);
            console.log(donate_details)
            result.push({
                programPubKey: item.pubkey,
                userPubKey: new PublicKey(donate_details.user),
                amount: donate_details.amount.toNumber()
            });
        } catch (err) {
            console.log(err);
        }
    });
    console.log(result)
    return result;
}


async function withdraw() {
  console.log("withdraw called")

  let donations = await getAllDonations()
         
  let transaction = new Transaction()
  donations.forEach((item) => {
      let instruction_data = []
      instruction_data.push({ pubkey: wallet.publicKey, isSigner: true })  
      instruction_data.push({ pubkey: item.programPubKey, isSigner: false, isWritable: true })
      console.log("withdraw. Instuction data: ", instruction_data)
      let instruction = new TransactionInstruction({
            keys: instruction_data,
            programId: programId,
            data: new Uint8Array([2])
      });
      transaction.add(instruction)
  });
 
  transaction.feePayer = wallet.publicKey
  transaction.recentBlockhash = (await connection.getRecentBlockhash()).blockhash

  let signed = await wallet.signTransaction(transaction)
  await broadcastSignedTransaction(signed)
}

