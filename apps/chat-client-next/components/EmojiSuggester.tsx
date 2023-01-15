import classNames from "classnames";
import { Emoji, search } from "node-emoji";
import { FC, useEffect, useState } from "react";

type SearchResult = {
  emoji: string;
  replaceWith: string;
};

type Props = {
  onSelect?(emoji: SearchResult): void;
  value: string;
};

const getEmojiSearchTerm = (str: string) => str.split(":").at(-1);

export const EmojiSuggester: FC<Props> = (props) => {
  const [suggestions, setSuggestions] = useState<Emoji[]>([]);
  const [focusIndex, setFocusIndex] = useState(0);
  const [searchTerm, setSearchTerm] = useState(getEmojiSearchTerm(props.value));

  useEffect(() => {
    if (!props.value.includes(":")) {
      setSuggestions([]);
      return;
    }
    const term = props.value.split(":").at(-1);
    setSearchTerm(term);
  }, [props.value]);

  useEffect(() => {
    if (!searchTerm) {
      setSuggestions([]);
      return;
    }
    const emojis = search(searchTerm).slice(0, 10);
    setSuggestions(emojis);
  }, [searchTerm]);

  useEffect(() => {
    const handleKeyDown = (e: KeyboardEvent) => {
      if (e.key === "ArrowDown" && focusIndex > 0) {
        e.preventDefault();
        return setFocusIndex((it) => (it -= 1));
      }
      if (e.key === "ArrowUp" && focusIndex < suggestions.length - 1) {
        e.preventDefault();
        return setFocusIndex((it) => (it += 1));
      }

      if (suggestions.length && (e.key === "Tab" || e.key === "Enter")) {
        e.preventDefault();
        e.stopPropagation();
        e.stopImmediatePropagation();
        props.onSelect?.({
          emoji: suggestions[focusIndex].emoji,
          replaceWith: `:${searchTerm}`,
        });
        setFocusIndex(0);
      }
    };

    window.addEventListener("keydown", handleKeyDown);
    return () => window.removeEventListener("keydown", handleKeyDown);
  }, [focusIndex, suggestions, searchTerm]);

  const onSelect = (emoji: Emoji) => {
    props.onSelect?.({
      emoji: emoji.emoji,
      replaceWith: `:${searchTerm}`,
    });
    setSuggestions([]);
    setFocusIndex(0);
  };

  if (suggestions.length === 0) {
    return <div></div>;
  }

  return (
    <div className="relative">
      <div className="absolute z-10 bg-white shadow-lg rounded p2 w-52 bottom-0 flex flex-col-reverse">
        {suggestions.map((it, index) => {
          return (
            <div
              onClick={() => onSelect(it)}
              key={it.key}
              onMouseEnter={() => setFocusIndex(index)}
              className={classNames({
                "overflow-x-scroll whitespace-nowrap rounded": true,
                "bg-gray-400": index === focusIndex,
              })}
            >
              <span className="text-xl">{it.emoji} </span> {it.key}
            </div>
          );
        })}
      </div>
    </div>
  );
};
