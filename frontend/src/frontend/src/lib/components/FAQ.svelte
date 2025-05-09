<script lang="ts">
  const faqItems = [
    {
      question: "What is Canister?",
      answer:
        "Canister is a decentralized cloud storage app where you can securely upload, store, and share documents — all powered by the ICP blockchain.",
    },
    {
      question: "How is Canister different from Dropbox or Google Drive?",
      answer:
        'Unlike traditional services, Canister gives you full control of your data in a personal "canister" — your own smart contract running on-chain, without centralized servers.',
    },
    {
      question: "Who owns the data I store on Canister?",
      answer:
        "You do. Your files are stored in your personal canister, and only you control access to it.",
    },
    {
      question: "How do I sign up or log in to Canister?",
      answer:
        "You log in using Internet Identity, a secure, passwordless system built on the ICP blockchain.",
    },
    {
      question: "Is my data encrypted?",
      answer:
        "Yes, files are encrypted before storage and stay encrypted inside your canister.",
    },
    {
      question: "Does storing data on Canister require cryptocurrency?",
      answer:
        "No crypto is required to get started. We handle the cost of storage (cycles) for you during the MVP phase.",
    },
    {
      question: "How much does Canister cost?",
      answer:
        "During the MVP, Canister is free for personal use. Pricing tiers will be introduced in later versions.",
    },
    {
      question: "How do I report a bug or give feedback?",
      answer:
        "You can use the in-app feedback option or email us at support@canister.co.",
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
