export type ButtonProps = {
  label: string;
  onClick(): void;
};

export const Button = (props: ButtonProps) => {
  const { onClick } = props;
  return (
    <button
      onClick={onClick}
      class="p-1 bg-blue-600 text-white rounded-md drop-shadow-md"
    >
      <label class="text-sm">{props.label}</label>
    </button>
  );
};
