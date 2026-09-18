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
        <CardTitle>Behind the scenes</CardTitle>
      </CardHeader>
      <CardContent className="space-y-3">
        {!speechState ? (
          <p className="text-sm text-muted-foreground">Starting up…</p>
        ) : (
          <div className="grid gap-x-10 sm:grid-cols-2">
            <KeyValueList
              items={[
                {
                  label: "Listening for words",
                  value:
                    speechState.recognizerId === "moonshine" ? "yes" : "no",
                },
                {
                  label: "Voice model ready",
                  value: speechState.modelLoaded ? "yes" : "no",
                },
                {
                  label: "Skips silence",
                  value: speechState.vadEnabled ? "yes" : "no",
                },
              ]}
            />
            <KeyValueList
              items={[
                {
                  label: "Times it heard speech",
                  value: speechState.segmentsSeen,
                },
                {
                  label: "Times it wrote words",
                  value: speechState.transcriptsGenerated,
                },
                { label: "Waiting for you", value: pendingCount },
              ]}
            />
          </div>
        )}

        {speechState?.recognizerId === "mock" ? (
          <p className="rounded-md border border-warning/30 bg-warning/10 px-3 py-2 text-xs text-warning">
            Selah is hearing the room but is not turning speech into words, so
            no Bible verses can be found. Turn on listening in Settings →
            Listening to enable it.
          </p>
        ) : null}

        {speechState?.lastError ? (
          <p className="text-xs text-destructive">
            Something went wrong: {speechState.lastError}
          </p>
        ) : null}
      </CardContent>
    </Card>
  );
}
