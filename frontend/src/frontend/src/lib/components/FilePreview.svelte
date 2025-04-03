<script lang="ts">
  export let file: { objectUrl: string; dataType: string };
  let previewStyle;
  let previewClass;

  const supportedDataTypes = [
    "application/pdf",
    "image/jpg",
    "image/jpeg",
    "image/png",
    "image/webp",
    "image/gif",
    "image/svg+xml",
    "image/tiff",
    "video/mp4",
    "video/webm",
    "video/ogg",
    "audio/mpeg",
    "audio/ogg",
    "audio/wav",
    "text/plain",
    "text/csv",
    "text/html",
  ];

  $: previewStyle =
    file.dataType === "application/pdf" ? "--bs-aspect-ratio: 141%" : "";
  $: previewClass = file.dataType === "application/pdf" ? "" : "";
</script>

<div class="flex items-center mb-3 w-full {previewClass}" style={previewStyle}>
  {#if supportedDataTypes.includes(file.dataType)}
    {#if file.dataType.startsWith("image/")}
      <img alt="Preview" src={file.objectUrl} class="max-w-full" />
    {:else if file.dataType === "application/pdf"}
      <object
        title="PDF Preview"
        type="application/pdf"
        data={file.objectUrl}
        class="w-full aspect-square"
      ></object>
    {:else if file.dataType.startsWith("video/")}
      <video controls class="w-full">
        <source src={file.objectUrl} type={file.dataType} />
        Your browser does not support the video tag.
      </video>
    {:else if file.dataType.startsWith("audio/")}
      <audio controls class="w-full">
        <source src={file.objectUrl} type={file.dataType} />
        Your browser does not support the audio tag.
      </audio>
    {:else if file.dataType.startsWith("text/")}
      <iframe src={file.objectUrl} class="w-full h-64" title="Text-File"
      ></iframe>
    {:else}
      <slot />
    {/if}
  {:else}
    <slot />
  {/if}
</div>
