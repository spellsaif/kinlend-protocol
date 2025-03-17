# Kinlend Protocol 🏦

Kinlend Protocol is a **peer-to-peer (P2P) lending platform** built on **Solana using Anchor Framework**. It allows **interest-free loans** while ensuring security through **over-collateralization**, protecting lenders from defaults. 

This decentralized protocol removes intermediaries, providing a **trustless** and **efficient** way to borrow and lend **USDC**.

---

## 🔥 Key Features

✅ **Interest-Free Loans** – Borrowers can request loans **without paying interest**.  
✅ **Over-Collateralization** – Borrowers must deposit collateral worth **at least 150%** of the loan amount.  
✅ **Decentralized Lending** – Any lender can fund loans, and once funded, the repayment **timer starts** (not immediate repayment).  
✅ **Service Fee Model** – Borrowers pay a **5% service fee**, distributed as follows:  
   - **4% goes to the lender** (incentive for lending).  
   - **1% goes to the protocol** (for ecosystem maintenance).  
✅ **Loan Request Cancellation** – Borrowers can cancel the request **if no lender has funded it**.  
✅ **Collateral Liquidation**:  
   - If the borrower **fails to repay within the deadline**, the lender **claims the collateral** (10% goes to the protocol).  
   - If the **collateral value drops to 110% of the loan amount**, the lender can **liquidate it immediately**, even if the repayment deadline hasn’t passed.  

---

## 🛠️ How It Works

### **🎯 Loan Lifecycle**

#### 1️⃣ **Loan Request**  
- The borrower creates a loan request, specifying:  
  - **Loan amount (in USDC)**  
  - **Collateral asset**  
  - **Repayment duration** (e.g., 30 days)  
  - **Over-collateralization (≥150%)**  
- The borrower **locks collateral** in the contract.  

#### 2️⃣ **Loan Funding**  
- Any lender can **accept and fund** the loan request.  
- As soon as a lender funds the request, the **repayment timer starts** (e.g., if the borrower set 30 days, the countdown begins from that moment).  

#### 3️⃣ **Borrower Receives Funds**  
- The borrower receives **USDC** and must repay it **before the deadline**.  

#### 4️⃣ **Repayment**  
- The borrower must **repay the loan amount + 5% service fee**.  
- Once repaid:  
  - **Collateral is returned to the borrower**.  
  - **Lender gets back their USDC + 4% fee**.  
  - **1% goes to the protocol**.  

#### 5️⃣ **Collateral Handling**  

##### 🔴 **If the Borrower Fails to Repay**  
- The lender can **claim the collateral**, but **10% of it goes to the protocol**.  

##### 🔥 **Early Liquidation (If Price Drops)**  
- If the **collateral’s value falls to 110% of the loan amount**, the lender can **liquidate it immediately**, even before the repayment deadline.  

---

## 📂 Documents 📜  

The `documents` folder contains:  

📌 **Architecture Diagram** 🏗️ (`documents/architecture.png`) – Visual overview of how the protocol functions.  
📌 **User Stories** 📖 (`documents/user_stories.md`) – Real-world borrower & lender scenarios.  

---

## 🚀 Deployment Guide (Solana Devnet)  

### ✅ **Prerequisites**  

Ensure you have the following installed:  

- **Rust & Cargo** (for Solana development):  
  ```bash
  curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh 
