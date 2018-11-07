import * as React from 'react';

import { Button } from '@material-ui/core';
import { ButtonProps } from '@material-ui/core/Button';

const inputStyle: React.CSSProperties = {
    display: 'none'
};

interface FilePickerProps extends ButtonProps {
    label: string;
    onFileSelected: (event: React.ChangeEvent<HTMLInputElement>) => void;
}

class FilePicker extends React.Component<FilePickerProps, {}> {
    private nativeElement: HTMLInputElement | null;

    public render() {
        return (
            <>
                <input
                    id="button-file"
                    multiple={false}
                    type="file"
                    style={inputStyle}
                    ref={input => this.nativeElement = input}
                    onChange={this.inputChanged}
                />
                <label htmlFor="button-file">
                    <Button component="span" style={this.props.style} variant={this.props.variant}>
                        {this.props.label}
                    </Button>
                </label>
            </>
        );
    }

    private inputChanged = (event: React.ChangeEvent<HTMLInputElement>) => {
        this.props.onFileSelected(event)
    }
}

export default FilePicker;