import { useState } from 'react';
import CodeEditor from '../components/editor/CodeEditor';

export default function Tools() {
    const [code, setCode] = useState('-- Write your script here\nprint("Hello World")');
    const [language, setLanguage] = useState<'javascript' | 'lua'>('lua');

    return (
        <div className="p-6 space-y-6">
            <div className="flex items-center justify-between">
                <h1 className="text-2xl font-bold text-gray-900">Tools & Scripts</h1>
                <div className="flex space-x-2">
                    <select
                        value={language}
                        onChange={(e) => setLanguage(e.target.value as 'javascript' | 'lua')}
                        className="block w-48 pl-3 pr-10 py-2 text-base border-gray-300 focus:outline-none focus:ring-blue-500 focus:border-blue-500 sm:text-sm rounded-md"
                    >
                        <option value="lua">Lua</option>
                        <option value="javascript">JavaScript</option>
                    </select>
                </div>
            </div>

            <div className="bg-white shadow rounded-lg overflow-hidden">
                <div className="px-6 py-4 border-b border-gray-200">
                    <h2 className="text-lg font-medium text-gray-900">Script Editor</h2>
                </div>

                <div className="w-full">
                    <CodeEditor
                        value={code}
                        onChange={(val) => setCode(val || '')}
                        language={language}
                        height="600px"
                    />
                </div>

                <div className="bg-gray-50 px-4 py-3 sm:px-6 flex justify-end">
                    <button
                        className="inline-flex items-center px-4 py-2 border border-transparent text-sm font-medium rounded-md shadow-sm text-white bg-blue-600 hover:bg-blue-700 focus:outline-none focus:ring-2 focus:ring-offset-2 focus:ring-blue-500"
                        onClick={() => alert(`Running ${language} script...`)}
                    >
                        Run Script
                    </button>
                </div>
            </div>
        </div>
    );
}
