import { KeyValueList } from "@/components/PageHeader";
import {
  Card,
  CardContent,
  CardHeader,
  CardTitle,
} from "@/components/ui/card";
import type { SpeechManagerState } from "@/types";

/**
 * Pipeline diagnostics: which recognizer is active, whether a model is
 * loaded, and how many segments/transcripts have flowed through.
 *
 * The mock-recognizer notice is deliberately prominent — the development
 * build must never look like it is transcribing when it is not.
 */
export default function PipelineDiagnosticsPanel({
  speechState,
  pendingCount,
}: {
  speechState: SpeechManagerState | null;
  pendingCount: number;
}) {
  return (
    <Card>
      <CardHeader>
        <CardTitle>Pipeline diagnostics</CardTitle>
      </CardHeader>
      <CardContent className="space-y-3">
        {!speechState ? (
          <p className="text-sm text-muted-foreground">
            Speech service is starting…
          </p>
        ) : (
          <div className="grid gap-x-10 sm:grid-cols-2">
            <KeyValueList
              items={[
                { label: "Recognizer", value: speechState.recognizerId },
                {
                  label: "Model loaded",
                  value: speechState.modelLoaded ? "yes" : "no",
                },
                {
                  label: "VAD",
                  value: speechState.vadEnabled ? "enabled" : "bypassed",
                },
              ]}
            />
            <KeyValueList
              items={[
                { label: "Speech segments", value: speechState.segmentsSeen },
                {
                  label: "Transcripts",
                  value: speechState.transcriptsGenerated,
                },
                { label: "Pending results", value: pendingCount },
              ]}
            />
          </div>
        )}

        {speechState?.recognizerId === "mock" ? (
          <p className="rounded-md border border-warning/30 bg-warning/10 px-3 py-2 text-xs text-warning">
            The development recognizer is active. It runs the real microphone →
            VAD pipeline but returns no text, so no Scripture can be detected.
            Build with the{" "}
            <code className="font-mono">whisper</code> cargo feature and install
            a model in Settings → Speech for real transcription.
          </p>
        ) : null}

        {speechState?.lastError ? (
          <p className="text-xs text-destructive">
            Last error: {speechState.lastError}
          </p>
        ) : null}
      </CardContent>
    </Card>
  );
}
