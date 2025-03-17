# Kinlend Protocol

**Kinlend Protocol** is a decentralized **peer-to-peer (P2P) lending platform** built on **Solana** using **Anchor Framework**. It enables **interest-free loans** with secure, over-collateralized borrowing and transparent lending.  

## 🚀 Features

✅ **Interest-Free Loans** – Borrowers can request loans **without paying interest**.  
✅ **Over-Collateralization** – Borrowers deposit collateral worth up to **150% of the loan amount**.  
✅ **Decentralized Lending** – Any lender can fund a loan using **USDC**, and repayment starts immediately.  
✅ **Service Fee Model** – Borrowers pay a **5% service fee**, distributed as follows:  
   - **4% goes to the lender** (as a reward).  
   - **1% goes to the protocol** (for sustainability).  
✅ **Default & Liquidation**:  
   - If a borrower **fails to repay**, the lender **claims the collateral** (with **10% going to the protocol**).  
   - If collateral value drops to **110% of the loan**, the lender can **liquidate it early**.  
✅ **Loan Request Cancellation** – Borrowers can cancel **if no lender accepts the request**.  

---

## 🔧 How It Works

### **🎯 Loan Lifecycle**

1️⃣ **Loan Request** → Borrower submits a request and deposits collateral.  
2️⃣ **Loan Funding** → Lenders fund the loan in **USDC**.  
3️⃣ **Fund Disbursement** → Borrower receives **USDC**, and repayment period begins.  
4️⃣ **Repayment** → Borrower repays the **loan + 5% fee**.  
5️⃣ **Lender Compensation** → Lender receives **USDC + 4% reward**.  
6️⃣ **Collateral Handling**:  
   - If **repaid** → Collateral is **returned to the borrower**.  
   - If **not repaid** → Lender **claims collateral** (10% goes to the protocol).  
   - If **collateral value drops to 110%** → Lender can **liquidate early**.  

---

## 📂 Documents

The `documents` folder contains:  
📌 **Architecture Diagram** 🏗️ (`documents/architecture.png`) – Visual structure of the protocol.  
📌 **User Stories** 📜 (`documents/user_stories.md`) – Real-world borrower/lender scenarios.  

---

## 🛠️ Deployment Guide (Solana Devnet)

To deploy **Kinlend Protocol** on **Solana Devnet**, follow these steps.

### ✅ **Prerequisites**

- Install **Rust & Cargo**:
  ```bash
  curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
  ```
