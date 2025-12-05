import Editor from '@monaco-editor/react';
import type { OnMount } from '@monaco-editor/react';
import { useRef } from 'react';

interface CodeEditorProps {
    value: string;
    onChange?: (value: string | undefined) => void;
    language?: 'lua' | 'javascript' | 'json' | 'typescript';
    readOnly?: boolean;
    height?: string;
    minimap?: boolean;
}

export default function CodeEditor({
    value,
    onChange,
    language = 'javascript',
    readOnly = false,
    height = '400px',
    minimap = false,
}: CodeEditorProps) {
    const editorRef = useRef(null);

    const handleEditorDidMount: OnMount = (editor) => {
        // @ts-expect-error - editor instance type
        editorRef.current = editor;
    };

    return (
        <div className="w-full overflow-hidden">
            <Editor
                height={height}
                defaultLanguage={language}
                language={language}
                value={value}
                onChange={onChange}
                theme="vs-dark"
                onMount={handleEditorDidMount}
                options={{
                    readOnly,
                    minimap: { enabled: minimap },
                    scrollBeyondLastLine: false,
                    fontSize: 14,
                    automaticLayout: true,
                    padding: { top: 16, bottom: 16 },
                }}
            />
        </div>
    );
}
