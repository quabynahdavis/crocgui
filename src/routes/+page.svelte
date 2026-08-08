<script lang="ts">
  import { ArrowUpFromLine, ArrowDownToLine } from "@lucide/svelte";
  import Button from "$lib/components/ui/button/button.svelte";
  import Card, { CardContent, CardDescription, CardHeader, CardTitle } from "$lib/components/ui/card/index.js";
</script>

<main class="flex min-h-dvh items-center justify-center p-4 pb-24 sm:pb-4">
  <div class="mx-auto flex w-full max-w-2xl flex-col items-center gap-8 text-center">
    <div class="space-y-2">
      <h1 class="text-3xl font-bold tracking-tight sm:text-4xl">croc-gui</h1>
      <p class="text-base text-muted-foreground sm:text-lg">
        Securely transfer files between computers using the croc CLI
      </p>
    </div>

    <div class="grid w-full grid-cols-1 gap-4 sm:grid-cols-1 sm:gap-6 md:grid-cols-2">
      <a href="/send" class="block" aria-label="Send files to another computer using a secret code phrase">
        <Card class="cursor-pointer transition-all hover:border-primary hover:shadow-lg">
          <CardHeader>
            <div class="flex justify-center">
              <div class="flex h-12 w-12 items-center justify-center rounded-full bg-primary/10">
                <ArrowUpFromLine class="h-6 w-6 text-primary" />
              </div>
            </div>
            <CardTitle class="text-center">Send Files</CardTitle>
            <CardDescription class="text-center">
              Share files with anyone using a secret code phrase
            </CardDescription>
          </CardHeader>
        </Card>
      </a>

      <a href="/receive" class="block" aria-label="Receive files from another computer by entering their code phrase">
        <Card class="cursor-pointer transition-all hover:border-primary hover:shadow-lg">
          <CardHeader>
            <div class="flex justify-center">
              <div class="flex h-12 w-12 items-center justify-center rounded-full bg-primary/10">
                <ArrowDownToLine class="h-6 w-6 text-primary" />
              </div>
            </div>
            <CardTitle class="text-center">Receive Files</CardTitle>
            <CardDescription class="text-center">
              Receive files by entering the code phrase from the sender
            </CardDescription>
          </CardHeader>
        </Card>
      </a>
    </div>
  </div>
</main>
