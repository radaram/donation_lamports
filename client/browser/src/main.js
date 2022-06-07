import Wallet from "@project-serum/sol-wallet-adapter";
import {
    Connection,
    SystemProgram,
    Transaction,
    PublicKey,
    TransactionInstruction,
    Keypair,
    sendAndConfirmTransaction
} from "@solana/web3.js";
import { deserialize, serialize } from "borsh";

//const cluster = "https://api.devnet.solana.com";
const cluster = "http://localhost:8899";
const connection = new Connection(cluster, "confirmed");
const programId = new PublicKey(
    "A7tb6TTYrjaSxYmL2PaETaFkEHq8jDAyGyJWLjdPTkyoal"
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
                ["timestamp", "u64"],
            ]
        }]]);

}


class WithdrawData {
   constructor(properties) {
        Object.keys(properties).forEach((key) => {
            this[key] = properties[key];
        });
    }
    static schema = new Map([[WithdrawData,
        {
            kind: "struct",
            fields: [
                ["timestamp", "u64"],
            ]
        }]]); 
}


async function donate(userPubKey, amount) {
    console.log("donate called")     
    let timestamp = (Math.floor(Date.now() / 1000)).toString()
    let donateDetails = new DonateDetails({
        user: wallet.publicKey.toBuffer(),
        amount: amount, 
        timestamp: timestamp,
    });
    console.log(donateDetails);
    let data = serialize(DonateDetails.schema, donateDetails);
    let dataToSend = new Uint8Array([1, ...data]);

    let [pda, bump] = await PublicKey.findProgramAddress(
        [
            wallet.publicKey.toBuffer(), 
            timestamp
        ],
        programId
    );

    const ix = new TransactionInstruction({
        keys: [
          {
            isSigner: true,
            isWritable: true,
            pubkey: wallet.publicKey,
          },
          {
            isSigner: false,
            isWritable: true,
            pubkey: pda,
          },
          {
            isSigner: false,
            isWritable: false,
            pubkey: SystemProgram.programId,
          },
        ],
        programId: programId,
        data: Buffer.from(dataToSend),
    });

    const tx = new Transaction();
    tx.add(ix);
    tx.feePayer = userPubKey;
    tx.recentBlockhash = (await connection.getRecentBlockhash()).blockhash
 
    console.log("start transaction")
    //
    let signed = await wallet.signTransaction(tx);
    await broadcastSignedTransaction(signed);
}


async function withdraw() {
  console.log("withdraw called")

  let donations = await getAllDonations()
         
  let transaction = new Transaction()
  donations.forEach((item) => {
      console.log(item)
      console.log(programId)
      let withdrawData = new WithdrawData({
          timestamp: item.timestamp
      });
      let data = serialize(WithdrawData.schema, withdrawData);
      let dataToSend = new Uint8Array([2, ...data]);

      let instruction_data = []
      instruction_data.push({ pubkey: wallet.publicKey, isSigner: true, isWritable: true  })  
      instruction_data.push({ pubkey: item.pdaPubKey, isSigner: false, isWritable: true })
      instruction_data.push({ pubkey: item.userPubKey, isSigner: false, isWritable: false })
      instruction_data.push({isSigner: false, isWritable: false, pubkey: SystemProgram.programId})
      console.log("withdraw. Instuction data: ", instruction_data)
      let instruction = new TransactionInstruction({
            keys: instruction_data,
            programId: programId,
            data: Buffer.from(dataToSend),
      });
      transaction.add(instruction)
  });
 
  transaction.feePayer = wallet.publicKey;
  transaction.recentBlockhash = (await connection.getRecentBlockhash()).blockhash;

  let signed = await wallet.signTransaction(transaction);
  await broadcastSignedTransaction(signed);
}


async function broadcastSignedTransaction(signed) {
  let signature = await connection.sendRawTransaction(signed.serialize());
  console.log("Submitted transaction " + signature + ", awaiting confirmation");
  await connection.confirmTransaction(signature);
  console.log("Transaction " + signature + " confirmed");
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
                pdaPubKey: item.pubkey,
                userPubKey: new PublicKey(donate_details.user),
                amount: donate_details.amount.toNumber(),
                timestamp: donate_details.timestamp,
            });
        } catch (err) {
            console.log(err);
        }
    });
    console.log(result);
    return result;
}

