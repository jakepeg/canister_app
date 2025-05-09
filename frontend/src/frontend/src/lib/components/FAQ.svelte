<script lang="ts">
  const faqItems = [
    {
      question: "What is Canister?",
      answer:
        "Canister is a decentralized cloud storage app where you can securely upload, store, and share documents — all powered by the ICP blockchain.",
    },
    {
      question: "What is the Internet Computer?",
      answer:
        "The Internet Computer (ICP) is a blockchain network that enables decentralized applications and services to run at web speed.",
    },
    {
      question: "What is an ICP Token?",
      answer:
        "ICP Tokens are the native cryptocurrency of the Internet Computer, used to pay for computation, store data, and participate in network governance.",
    },
    {
      question: "What is Internet Identity?",
      answer:
        "Internet Identity is a secure, passwordless authentication system built on the ICP blockchain.",
    },
    {
      question: "What is Decentralized Cloud Storage?",
      answer:
        "Decentralized cloud storage stores data on a distributed network rather than centralized servers, giving users more control over their data.",
    },
    {
      question:
        "What is the difference between personal and enterprise Canisters?",
      answer:
        "Personal Canisters are designed for individual use, while enterprise Canisters offer enhanced features like multi-user access and data management.",
    },
    {
      question: "How do I log in?",
      answer:
        "You log in using Internet Identity, a secure, passwordless system built on the ICP blockchain.",
    },
    {
      question: "How do I create a Canister?",
      answer:
        "Click the 'Create Canister' button, enter a name, and click 'Create Canister'. You need at least 1 ICP to create a Canister.",
    },
    {
      question: "How do I add funds?",
      answer:
        "Click the profile icon, copy your account ID, purchase ICP on an exchange, and send 1 or more ICP to your account ID.",
    },
    {
      question: "How do I top-up a Canister?",
      answer:
        "On the Canister list screen, click on the 3 dots at the top left of the Canister card, select 'Topup Cycles', enter the amount, and click 'Top Up'.",
    },
    {
      question: "How do I start or stop a Canister?",
      answer:
        "On the Canister list screen, click on the 3 dots at the top left of the Canister card, select 'Start' or 'Stop'.",
    },
    {
      question: "How do I delete a Canister?",
      answer:
        "On the Canister list screen, click on the 3 dots at the top left of the Canister card, select 'Delete' and confirm.",
    },
    {
      question: "How do I rename a Canister?",
      answer:
        "On the Canister list screen, click on the 3 dots at the top left of the Canister card, select 'Rename', enter a new name, and confirm.",
    },
    {
      question: "How do I upload files?",
      answer: "Use the upload button in your Canister interface to add files.",
    },
    {
      question: "How do I request files?",
      answer:
        "Click the 'Request' button, enter the file name, and click 'Create Request'. Share the generated link with others to receive files.",
    },
    {
      question: "How do I delete files?",
      answer:
        "On the file preview screen, click the delete (trashcan) icon and confirm the action.",
    },
    {
      question: "How do I rename files?",
      answer:
        "On the file preview screen, click the edit (pencil) icon, change the name, and confirm.",
    },
    {
      question: "How do I recover my account?",
      answer: "Use your recovery passphrase to regain access.",
    },
    {
      question: "How can I get in touch?",
      answer: "For support, reach out via email at support@canister.co.",
    },
  ];
</script>

<section class="max-w-3xl mx-auto px-4 py-12">
  <h1 class="text-3xl font-bold mb-8 text-center">
    Frequently Asked Questions
  </h1>

  <div class="space-y-8">
    {#each faqItems as item}
      <div>
        <h2 class="text-xl font-semibold mb-2">{item.question}</h2>
        <p class="text-gray-400">{item.answer}</p>
      </div>
    {/each}
  </div>
</section>
