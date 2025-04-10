<script>
  import LinkedInIcon from "../icons/linkedin.svelte";
  import XIcon from "../icons/x-logo.svelte";
  import Badge from "../icons/on-chain.svelte";
</script>

<div class="bg-background flex flex-col items-center px-6">
  <!-- Main Content -->
  <div class="flex flex-col items-center text-center mt-8 max-w-3xl">
    <h1 class="text-4xl font-bold">Store, Share & Collect Documents</h1>
    <h2 class="text-lg mt-2 text-gray-400">
      Take control with an Enterprise-Grade Canister, built for trust and
      compliant by design.
    </h2>
  </div>

  <!-- Screenshot Image Full Width (Same as Feature Boxes Row) -->
  <div class="mt-12 max-w-5xl">
    <div class="">
      <img
        src="/canister-preview.png"
        alt="Canister Preview"
        class="rounded-xl w-full glow-image"
      />
    </div>
  </div>

  <!-- Demo Notice Section -->
  <div class="mt-12 text-center text-gray-400">
    <h2 class="text-lg">
      This is a Demo version of our MVP, do not upload confidential data.<br />
      Big things are coming:
    </h2>
  </div>

  <!-- Feature Boxes -->
  <div class="grid grid-cols-1 md:grid-cols-3 gap-6 mt-16 max-w-5xl">
    <div class="glow-box">
      <h3 class="text-xl font-semibold">Scalability</h3>
      <p class="text-gray-300 mt-2">
        Deploy Canisters at scale with custom installations, white-labeled
        interfaces, and workflow templates tailored to your organization's
        needs.
      </p>
    </div>

    <div class="glow-box">
      <h3 class="text-xl font-semibold">Colaboration</h3>
      <p class="text-gray-300 mt-2">
        Collect and share documents without messy email chains. With smart
        permissions and ERP integration, your team stays aligned and in control.
      </p>
    </div>

    <div class="glow-box">
      <h3 class="text-xl font-semibold">Compliance</h3>
      <p class="text-gray-300 mt-2">
        Operate with confidence using infrastructure that supports GDPR and data
        privacy by default. Built for trust, from the ground up.
      </p>
    </div>
  </div>

  <!-- Social & Badge Section -->

  <div class="mt-12 text-center text-gray-400">
    <h2 class="text-lg">
      Follow us for updates and news on our upcoming launch.<br />
    </h2>
  </div>

  <div class="mt-6 text-center">
    <div class="flex gap-4 justify-center">
      <a
        href="https://www.linkedin.com/company/canister-cloud/"
        target="_blank"
      >
        <span class="h-6 w-6">
          <LinkedInIcon />
        </span>
      </a>
      <a href="https://x.com/CanisterCloud" target="_blank">
        <span class="h-6 w-6">
          <XIcon />
        </span>
      </a>
    </div>
  </div>
  <!-- <div class="mt-6 text-center">
    <a href="https://internetcomputer.org/" target="_blank">
      <Badge />
    </a>
  </div> -->
</div>
