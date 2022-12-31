import { Show } from "solid-js";

type InputProps = {
  label?: string;
  value?: string;
  onInput(value: string): void;
  onEnter?(): void;
};

export const Input = (props: InputProps) => {
  const id = Math.floor(Math.random() * 10_000_000).toString();
  const { onInput, onEnter } = props;

  return (
    <div class="flex flex-col w-full">
      <Show when={props.label}>
        <label class="text-xs" for={id}>
          {props.label}
        </label>
      </Show>
      <input
        class="border rounded outline-1 outline-blue-300 p-1 w-full"
        id={id}
        type="text"
        value={props.value ?? ""}
        onInput={(e) => onInput(e.currentTarget.value)}
        onKeyUp={(e) => e.key === "Enter" && onEnter?.()}
      />
    </div>
  );
};
