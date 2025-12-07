import { Dialog, Transition } from '@headlessui/react';
import { Fragment, useState, useEffect } from 'react';
import { X, Play, Loader2 } from 'lucide-react';
import type { TemplateDetails, ParameterSchema } from '../../api/types';
import clsx from 'clsx';

interface LaunchModalProps {
    isOpen: boolean;
    onClose: () => void;
    template: TemplateDetails | null;
    onLaunch: (id: string, config: Record<string, any>) => Promise<void>;
}

export function LaunchModal({ isOpen, onClose, template, onLaunch }: LaunchModalProps) {
    const [config, setConfig] = useState<Record<string, any>>({});
    const [isLaunching, setIsLaunching] = useState(false);
    const [error, setError] = useState<string | null>(null);

    useEffect(() => {
        if (isOpen && template) {
            // Initialize defaults
            const defaults: Record<string, any> = {};
            template.schema.parameters.forEach(p => {
                if (p.default !== undefined && p.default !== null) {
                    defaults[p.name] = p.default;
                } else if (p.type === 'boolean') {
                    defaults[p.name] = false;
                }
            });
            setConfig(defaults);
            setError(null);
        }
    }, [isOpen, template]);

    const handleSubmit = async (e: React.FormEvent) => {
        e.preventDefault();
        if (!template) return;

        // Validation
        for (const p of template.schema.parameters) {
            const val = config[p.name];

            // Required check
            if (p.required && (val === undefined || val === '' || val === null)) {
                setError(`Parameter '${p.name}' is required`);
                return;
            }

            // Numeric constraints check
            if ((p.type === 'integer' || p.type === 'number') && val !== undefined && val !== '') {
                const numVal = Number(val);
                if (p.constraints?.min !== undefined && numVal < p.constraints.min) {
                    setError(`Parameter '${p.name}' must be at least ${p.constraints.min}`);
                    return;
                }
                if (p.constraints?.max !== undefined && numVal > p.constraints.max) {
                    setError(`Parameter '${p.name}' must be at most ${p.constraints.max}`);
                    return;
                }
            }
        }

        setIsLaunching(true);
        setError(null);

        try {
            // Filter out null/undefined values and empty strings to keep payload clean
            const cleanConfig = Object.entries(config).reduce((acc, [key, value]) => {
                if (value !== null && value !== undefined && value !== '') {
                    acc[key] = value;
                }
                return acc;
            }, {} as Record<string, any>);

            await onLaunch(template.metadata.id, cleanConfig);
            onClose();
        } catch (err: any) {
            setError(err.message || 'Failed to launch template');
        } finally {
            setIsLaunching(false);
        }
    };

    const handleInputChange = (key: string, value: any) => {
        setConfig(prev => ({ ...prev, [key]: value }));
    };

    // Mock data for providers and models (until Registry API is ready)
    const PROVIDER_OPTIONS = ['ollama', 'openai', 'anthropic', 'candle'];
    const MODEL_OPTIONS = ['gpt-4-turbo', 'gpt-3.5-turbo', 'claude-3-opus', 'llama3-8b', 'mistral-7b'];

    const renderField = (param: ParameterSchema) => {
        const value = config[param.name] ?? '';

        // Specialized Renderers for specific semantic fields
        if (param.name === 'provider_name') {
            return (
                <div key={param.name}>
                    <label htmlFor={param.name} className="block text-sm font-medium text-gray-700">
                        {param.name} {param.required && <span className="text-red-500">*</span>}
                    </label>
                    <select
                        id={param.name}
                        value={value}
                        onChange={(e) => handleInputChange(param.name, e.target.value)}
                        className="mt-1 block w-full rounded-md border-gray-300 shadow-sm focus:border-blue-500 focus:ring-blue-500 sm:text-sm border p-2"
                    >
                        <option value="">Default (Auto)</option>
                        {PROVIDER_OPTIONS.map((val) => (
                            <option key={val} value={val}>{val}</option>
                        ))}
                    </select>
                    <p className="mt-1 text-xs text-gray-500">{param.description || 'Select inference provider'}</p>
                </div>
            );
        }

        if (param.name === 'model') {
            return (
                <div key={param.name}>
                    <label htmlFor={param.name} className="block text-sm font-medium text-gray-700">
                        {param.name} {param.required && <span className="text-red-500">*</span>}
                    </label>
                    <select
                        id={param.name}
                        value={value}
                        onChange={(e) => handleInputChange(param.name, e.target.value)}
                        className="mt-1 block w-full rounded-md border-gray-300 shadow-sm focus:border-blue-500 focus:ring-blue-500 sm:text-sm border p-2"
                    >
                        <option value="">Default (Template Defined)</option>
                        {MODEL_OPTIONS.map((val) => (
                            <option key={val} value={val}>{val}</option>
                        ))}
                    </select>
                    <p className="mt-1 text-xs text-gray-500">{param.description || 'Override default model'}</p>
                </div>
            );
        }

        if (param.type === 'boolean') {
            return (
                <div key={param.name} className="flex items-center">
                    <input
                        type="checkbox"
                        id={param.name}
                        checked={!!config[param.name]}
                        onChange={(e) => handleInputChange(param.name, e.target.checked)}
                        className="h-4 w-4 text-blue-600 focus:ring-blue-500 border-gray-300 rounded"
                    />
                    <label htmlFor={param.name} className="ml-2 block text-sm text-gray-900">
                        {param.name}
                        {param.required && <span className="text-red-500">*</span>}
                        {param.description && <span className="text-gray-500 ml-2 text-xs">({param.description})</span>}
                    </label>
                </div>
            );
        }

        // Add Select support if allowed_values exists
        if (param.constraints?.allowed_values) {
            return (
                <div key={param.name}>
                    <label htmlFor={param.name} className="block text-sm font-medium text-gray-700">
                        {param.name} {param.required && <span className="text-red-500">*</span>}
                    </label>
                    <select
                        id={param.name}
                        value={value}
                        onChange={(e) => handleInputChange(param.name, e.target.value)}
                        className="mt-1 block w-full rounded-md border-gray-300 shadow-sm focus:border-blue-500 focus:ring-blue-500 sm:text-sm border p-2"
                    >
                        {/* If not required, add empty option */}
                        {!param.required && <option value="">Select...</option>}
                        {param.constraints.allowed_values.map((val: any) => (
                            <option key={val} value={val}>{val}</option>
                        ))}
                    </select>
                    {param.description && <p className="mt-1 text-xs text-gray-500">{param.description}</p>}
                </div>
            )
        }

        const isNum = param.type === 'integer' || param.type === 'number';

        return (
            <div key={param.name}>
                <label htmlFor={param.name} className="block text-sm font-medium text-gray-700">
                    {param.name} {param.required && <span className="text-red-500">*</span>}
                </label>
                <input
                    type={isNum ? 'number' : 'text'}
                    id={param.name}
                    value={value}
                    min={isNum ? param.constraints?.min : undefined}
                    max={isNum ? param.constraints?.max : undefined}
                    onChange={(e) => {
                        const val = e.target.value;
                        if (isNum) {
                            handleInputChange(param.name, val === '' ? '' : Number(val));
                        } else {
                            handleInputChange(param.name, val);
                        }
                    }}
                    className="mt-1 block w-full rounded-md border-gray-300 shadow-sm focus:border-blue-500 focus:ring-blue-500 sm:text-sm border p-2"
                    placeholder={param.description}
                />
                {isNum && (param.constraints?.min !== undefined || param.constraints?.max !== undefined) && (
                    <p className="mt-1 text-xs text-gray-400">
                        Range: {param.constraints?.min ?? 'Any'} - {param.constraints?.max ?? 'Any'}
                    </p>
                )}
            </div>
        );
    };

    return (
        <Transition appear show={isOpen} as={Fragment}>
            <Dialog as="div" className="relative z-10" onClose={onClose}>
                <Transition.Child
                    as={Fragment}
                    enter="ease-out duration-300"
                    enterFrom="opacity-0"
                    enterTo="opacity-100"
                    leave="ease-in duration-200"
                    leaveFrom="opacity-100"
                    leaveTo="opacity-0"
                >
                    <div className="fixed inset-0 bg-black bg-opacity-25" />
                </Transition.Child>

                <div className="fixed inset-0 overflow-y-auto">
                    <div className="flex min-h-full items-center justify-center p-4 text-center">
                        <Transition.Child
                            as={Fragment}
                            enter="ease-out duration-300"
                            enterFrom="opacity-0 scale-95"
                            enterTo="opacity-100 scale-100"
                            leave="ease-in duration-200"
                            leaveFrom="opacity-100 scale-100"
                            leaveTo="opacity-0 scale-95"
                        >
                            <Dialog.Panel className="w-full max-w-md transform overflow-hidden rounded-2xl bg-white p-6 text-left align-middle shadow-xl transition-all">
                                <div className="flex justify-between items-center mb-4">
                                    <Dialog.Title as="h3" className="text-lg font-medium leading-6 text-gray-900">
                                        Launch {template?.metadata.name}
                                    </Dialog.Title>
                                    <button onClick={onClose} className="text-gray-400 hover:text-gray-500">
                                        <X className="h-5 w-5" />
                                    </button>
                                </div>

                                <form onSubmit={handleSubmit} className="space-y-4">
                                    <div className="bg-gray-50 p-4 rounded-md text-sm text-gray-600 mb-4 max-h-32 overflow-y-auto">
                                        {template?.metadata.description}
                                    </div>

                                    <div className="space-y-3">
                                        {(template?.schema.parameters || []).map(renderField)}
                                        {template?.schema.parameters.length === 0 && (
                                            <div className="text-sm text-gray-500 italic p-4 bg-gray-50 rounded text-center">
                                                No configuration options available.
                                            </div>
                                        )}
                                    </div>

                                    {error && (
                                        <div className="text-sm text-red-600 bg-red-50 p-2 rounded">
                                            {error}
                                        </div>
                                    )}

                                    <div className="mt-6 flex justify-end space-x-3">
                                        <button
                                            type="button"
                                            onClick={onClose}
                                            className="inline-flex justify-center rounded-md border border-gray-300 bg-white px-4 py-2 text-sm font-medium text-gray-700 hover:bg-gray-50 focus:outline-none focus:ring-2 focus:ring-blue-500 focus:ring-offset-2"
                                        >
                                            Cancel
                                        </button>
                                        <button
                                            type="submit"
                                            disabled={isLaunching}
                                            className={clsx(
                                                "inline-flex justify-center rounded-md border border-transparent bg-blue-600 px-4 py-2 text-sm font-medium text-white hover:bg-blue-700 focus:outline-none focus:ring-2 focus:ring-blue-500 focus:ring-offset-2 disabled:opacity-50 disabled:cursor-not-allowed items-center",
                                                isLaunching && "opacity-75 cursor-not-allowed"
                                            )}
                                        >
                                            {isLaunching ? (
                                                <>
                                                    <Loader2 className="animate-spin -ml-1 mr-2 h-4 w-4" />
                                                    Starting...
                                                </>
                                            ) : (
                                                <>
                                                    <Play className="-ml-1 mr-2 h-4 w-4" />
                                                    Start Session
                                                </>
                                            )}
                                        </button>
                                    </div>
                                </form>
                            </Dialog.Panel>
                        </Transition.Child>
                    </div>
                </div>
            </Dialog>
        </Transition>
    );
}
